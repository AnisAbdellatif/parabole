//! libmpv render API on top of eframe's glow context: mpv renders into an FBO texture that egui
//! shows as a normal image, so egui controls can be drawn over the video.

use eframe::glow::{self, HasContext as _};
use egui::{Color32, Pos2, Rect, TextureId};
use libmpv2::Mpv;
use libmpv2::render::{OpenGLInitParams, RenderContext, RenderParam, RenderParamApiType};
use std::ffi::{CStr, CString, c_void};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// Address of a `&dyn Fn` that lives on the caller's stack for the duration of context creation.
pub(crate) struct ProcAddress(usize);

pub(crate) type Loader<'a> = &'a dyn Fn(&CStr) -> *const c_void;

impl ProcAddress {
    pub(crate) fn new(loader: &Loader<'_>) -> Self {
        Self(loader as *const Loader<'_> as usize)
    }
}

pub(crate) fn proc_address(ctx: &ProcAddress, name: &str) -> *mut c_void {
    let Ok(name) = CString::new(name) else {
        return std::ptr::null_mut();
    };
    // SAFETY: mpv only calls this inside `create_render_context`, while the loader is alive.
    let loader = unsafe { &*(ctx.0 as *const Loader<'_>) };
    loader(&name).cast_mut()
}

struct Target {
    fbo: glow::Framebuffer,
    tex: glow::Texture,
    id: TextureId,
    alloc: (i32, i32),
}

/// State shared with egui paint callbacks.
pub struct Shared {
    render: Mutex<Option<RenderContext<'static>>>,
    dirty: AtomicBool,
    last_size: Mutex<(i32, i32)>,
}

// SAFETY: the render context is only used under the mutex, and every GL call it makes happens
// inside egui paint callbacks, which run on the UI thread that owns the GL context.
unsafe impl Send for Shared {}
unsafe impl Sync for Shared {}

pub struct VideoOutput {
    pub shared: Arc<Shared>,
    target: Mutex<Option<Target>>,
}

impl VideoOutput {
    /// `mpv` must outlive the returned output; `release` must run before `mpv` is dropped.
    pub fn new(mpv: &Mpv, loader: Loader<'_>, ctx: &egui::Context) -> libmpv2::Result<VideoOutput> {
        let mut render = mpv.create_render_context([
            RenderParam::ApiType(RenderParamApiType::OpenGl),
            RenderParam::InitParams(OpenGLInitParams {
                get_proc_address: proc_address,
                ctx: ProcAddress::new(&loader),
            }),
        ])?;
        let shared = Arc::new(Shared {
            render: Mutex::new(None),
            dirty: AtomicBool::new(true),
            last_size: Mutex::new((0, 0)),
        });
        let (flag, repaint) = (Arc::clone(&shared), ctx.clone());
        render.set_update_callback(move || {
            flag.dirty.store(true, Ordering::Release);
            repaint.request_repaint();
        });
        // SAFETY: the owner keeps the Mpv alive until `release` drops this context.
        let render: RenderContext<'static> = unsafe { std::mem::transmute(render) };
        *shared.render.lock().unwrap_or_else(|e| e.into_inner()) = Some(render);
        Ok(VideoOutput {
            shared,
            target: Mutex::new(None),
        })
    }

    /// Drops the render context; call with the GL context current and before the Mpv is dropped.
    pub fn release(&self) {
        let ctx = self
            .shared
            .render
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .take();
        drop(ctx);
    }

    /// Draws the current video frame into `rect`, letterboxed by mpv itself.
    pub fn paint(&self, ui: &egui::Ui, frame: &mut eframe::Frame, rect: Rect) {
        let ppp = ui.pixels_per_point();
        let used = (
            ((rect.width() * ppp).round() as i32).max(1),
            ((rect.height() * ppp).round() as i32).max(1),
        );
        let Some((fbo, tex_id, alloc)) = self.ensure_target(frame, used) else {
            return;
        };
        let shared = Arc::clone(&self.shared);
        let callback = eframe::egui_glow::CallbackFn::new(move |_, _| {
            let mut last = shared.last_size.lock().unwrap_or_else(|e| e.into_inner());
            let redraw = shared.dirty.swap(false, Ordering::AcqRel) || *last != used;
            if !redraw {
                return;
            }
            *last = used;
            if let Some(r) = shared
                .render
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .as_ref()
            {
                let _ = r.render::<()>(fbo, used.0, used.1, false);
            }
        });
        let painter = ui.painter();
        painter.add(egui::PaintCallback {
            rect,
            callback: Arc::new(callback),
        });
        let uv = Rect::from_min_max(
            Pos2::ZERO,
            Pos2::new(
                used.0 as f32 / alloc.0 as f32,
                used.1 as f32 / alloc.1 as f32,
            ),
        );
        painter.image(tex_id, rect, uv, Color32::WHITE);
    }

    /// Returns the FBO, its egui texture and the allocated size, growing the texture when needed.
    fn ensure_target(
        &self,
        frame: &mut eframe::Frame,
        used: (i32, i32),
    ) -> Option<(i32, TextureId, (i32, i32))> {
        let gl = frame.gl()?.clone();
        let mut slot = self.target.lock().unwrap_or_else(|e| e.into_inner());
        let alloc = alloc_size(used);
        match slot.as_mut() {
            Some(t) if t.alloc.0 >= used.0 && t.alloc.1 >= used.1 => {}
            // Same GL texture and FBO, new storage: the egui texture id stays valid and nothing leaks
            // (eframe has no API to free or replace a registered native texture).
            Some(t) => {
                unsafe { allocate_storage(&gl, t.tex, alloc) };
                t.alloc = alloc;
            }
            None => {
                let (fbo, tex) = unsafe { create_target(&gl, alloc) }?;
                let id = frame.register_native_glow_texture(tex);
                *slot = Some(Target {
                    fbo,
                    tex,
                    id,
                    alloc,
                });
            }
        }
        let t = slot.as_ref()?;
        Some((t.fbo.0.get() as i32, t.id, t.alloc))
    }
}

/// Texture size to allocate for a pane of `used` pixels: at least 512x288, rounded up to 256 steps.
pub(crate) fn alloc_size((w, h): (i32, i32)) -> (i32, i32) {
    (round_up(w.max(512)), round_up(h.max(288)))
}

fn round_up(v: i32) -> i32 {
    (v + 255) / 256 * 256
}

/// # Safety
/// The GL context behind `gl` must be current.
pub(crate) unsafe fn allocate_storage(gl: &glow::Context, tex: glow::Texture, (w, h): (i32, i32)) {
    unsafe {
        gl.bind_texture(glow::TEXTURE_2D, Some(tex));
        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            glow::RGBA8 as i32,
            w,
            h,
            0,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            glow::PixelUnpackData::Slice(None),
        );
        gl.bind_texture(glow::TEXTURE_2D, None);
    }
}

/// # Safety
/// The GL context behind `gl` must be current.
pub(crate) unsafe fn create_target(
    gl: &glow::Context,
    size: (i32, i32),
) -> Option<(glow::Framebuffer, glow::Texture)> {
    unsafe {
        let tex = gl.create_texture().ok()?;
        allocate_storage(gl, tex, size);
        gl.bind_texture(glow::TEXTURE_2D, Some(tex));
        for (param, value) in [
            (glow::TEXTURE_MIN_FILTER, glow::LINEAR),
            (glow::TEXTURE_MAG_FILTER, glow::LINEAR),
            (glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE),
            (glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE),
        ] {
            gl.tex_parameter_i32(glow::TEXTURE_2D, param, value as i32);
        }
        let Ok(fbo) = gl.create_framebuffer() else {
            gl.delete_texture(tex);
            return None;
        };
        gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo));
        gl.framebuffer_texture_2d(
            glow::FRAMEBUFFER,
            glow::COLOR_ATTACHMENT0,
            glow::TEXTURE_2D,
            Some(tex),
            0,
        );
        gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        gl.bind_texture(glow::TEXTURE_2D, None);
        Some((fbo, tex))
    }
}

#[cfg(test)]
mod tests {
    use super::alloc_size;

    #[test]
    fn alloc_has_minimum_and_rounds_up() {
        assert_eq!(alloc_size((1, 1)), (512, 512));
        assert_eq!(alloc_size((513, 300)), (768, 512));
        assert_eq!(alloc_size((1280, 720)), (1280, 768));
    }
}
