//! mpv rendered into a GL texture that Slint's femtovg renderer draws as an `Image`.
use super::super::{AppWindow, PlayerApi};
use crate::player::embed::render::{
    Loader, ProcAddress, allocate_storage, create_target, proc_address,
};
use eframe::glow::{self, HasContext as _};
use libmpv2::Mpv;
use libmpv2::render::{OpenGLInitParams, RenderContext, RenderParam, RenderParamApiType};
use slint::{
    BorrowedOpenGLTextureBuilder, BorrowedOpenGLTextureOrigin, ComponentHandle, GraphicsAPI,
    RenderingState,
};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// GL objects and the mpv render context; lives inside the rendering notifier.
struct Video {
    gl: glow::Context,
    render: Option<RenderContext<'static>>,
    fbo: glow::Framebuffer,
    tex: glow::Texture,
    size: (i32, i32),
    dirty: Arc<AtomicBool>,
    image_size: (i32, i32),
}

/// GL state mpv's renderer may change and femtovg relies on.
struct Saved {
    fbo: Option<glow::Framebuffer>,
    program: Option<glow::Program>,
    vao: Option<glow::VertexArray>,
    array_buffer: Option<glow::Buffer>,
    texture: Option<glow::Texture>,
    active_unit: i32,
    viewport: [i32; 4],
    enabled: Vec<(u32, bool)>,
}

const TOGGLES: [u32; 5] = [
    glow::BLEND,
    glow::SCISSOR_TEST,
    glow::DEPTH_TEST,
    glow::STENCIL_TEST,
    glow::CULL_FACE,
];

fn id(v: i32) -> Option<NonZeroU32> {
    NonZeroU32::new(v as u32)
}

impl Saved {
    fn capture(gl: &glow::Context) -> Self {
        // SAFETY: the GL context is current inside the rendering notifier.
        unsafe {
            let mut viewport = [0; 4];
            gl.get_parameter_i32_slice(glow::VIEWPORT, &mut viewport);
            let active_unit = gl.get_parameter_i32(glow::ACTIVE_TEXTURE);
            gl.active_texture(glow::TEXTURE0);
            Self {
                fbo: id(gl.get_parameter_i32(glow::FRAMEBUFFER_BINDING))
                    .map(glow::NativeFramebuffer),
                program: id(gl.get_parameter_i32(glow::CURRENT_PROGRAM)).map(glow::NativeProgram),
                vao: id(gl.get_parameter_i32(glow::VERTEX_ARRAY_BINDING))
                    .map(glow::NativeVertexArray),
                array_buffer: id(gl.get_parameter_i32(glow::ARRAY_BUFFER_BINDING))
                    .map(glow::NativeBuffer),
                texture: id(gl.get_parameter_i32(glow::TEXTURE_BINDING_2D))
                    .map(glow::NativeTexture),
                active_unit,
                viewport,
                enabled: TOGGLES.iter().map(|&c| (c, gl.is_enabled(c))).collect(),
            }
        }
    }

    fn restore(self, gl: &glow::Context) {
        // SAFETY: as in `capture`.
        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, self.fbo);
            gl.use_program(self.program);
            gl.bind_vertex_array(self.vao);
            gl.bind_buffer(glow::ARRAY_BUFFER, self.array_buffer);
            gl.bind_texture(glow::TEXTURE_2D, self.texture);
            gl.active_texture(self.active_unit as u32);
            let [x, y, w, h] = self.viewport;
            gl.viewport(x, y, w, h);
            for (cap, on) in self.enabled {
                if on {
                    gl.enable(cap);
                } else {
                    gl.disable(cap);
                }
            }
        }
    }
}

impl Video {
    fn new(
        gl: glow::Context,
        mpv: &Mpv,
        loader: Loader<'_>,
        win: slint::Weak<AppWindow>,
    ) -> Option<Self> {
        let mut render = mpv
            .create_render_context([
                RenderParam::ApiType(RenderParamApiType::OpenGl),
                RenderParam::InitParams(OpenGLInitParams {
                    get_proc_address: proc_address,
                    ctx: ProcAddress::new(&loader),
                }),
            ])
            .ok()?;
        let dirty = Arc::new(AtomicBool::new(true));
        let flag = Arc::clone(&dirty);
        render.set_update_callback(move || {
            flag.store(true, Ordering::Release);
            let _ = win.upgrade_in_event_loop(|w| w.window().request_redraw());
        });
        // SAFETY: `Mpv` is boxed in `MpvCore`, which outlives this context (dropped in teardown).
        let render: RenderContext<'static> = unsafe { std::mem::transmute(render) };
        let size = (2, 2);
        // SAFETY: GL context is current during RenderingSetup.
        let (fbo, tex) = unsafe { create_target(&gl, size) }?;
        Some(Self {
            gl,
            render: Some(render),
            fbo,
            tex,
            size,
            dirty,
            image_size: (0, 0),
        })
    }

    /// Renders a frame when mpv has one (or the pane resized); returns a fresh image on resize.
    fn frame(&mut self, used: (i32, i32)) -> Option<slint::Image> {
        let resized = used != self.size;
        if !resized && !self.dirty.swap(false, Ordering::AcqRel) {
            return None;
        }
        self.dirty.store(false, Ordering::Release);
        let saved = Saved::capture(&self.gl);
        if resized {
            // SAFETY: GL context current.
            unsafe { allocate_storage(&self.gl, self.tex, used) };
            self.size = used;
        }
        if let Some(r) = &self.render {
            let _ = r.render::<()>(self.fbo.0.get() as i32, used.0, used.1, false);
        }
        saved.restore(&self.gl);
        if self.image_size == used {
            return None;
        }
        self.image_size = used;
        let size = [used.0 as u32, used.1 as u32].into();
        // SAFETY: the texture is a live 2D RGBA texture of `used` size owned by this struct.
        Some(
            unsafe { BorrowedOpenGLTextureBuilder::new_gl_2d_rgba_texture(self.tex.0, size) }
                .origin(BorrowedOpenGLTextureOrigin::BottomLeft)
                .build(),
        )
    }

    /// Drops the render context first, then the GL objects.
    fn release(mut self) {
        drop(self.render.take());
        // SAFETY: GL context current during teardown.
        unsafe {
            self.gl.delete_framebuffer(self.fbo);
            self.gl.delete_texture(self.tex);
        }
    }
}

/// Registers the notifier. `mpv` must stay valid until the teardown notification; `on_fail` gets the reason.
pub fn install(
    win: &AppWindow,
    mpv: *const Mpv,
    on_ready: impl Fn() + 'static,
    on_fail: impl Fn(String) + 'static,
) -> Result<(), slint::SetRenderingNotifierError> {
    let weak = win.as_weak();
    let mut video: Option<Video> = None;
    win.window()
        .set_rendering_notifier(move |state, api| match state {
            RenderingState::RenderingSetup => {
                let GraphicsAPI::NativeOpenGL { get_proc_address } = api else {
                    return on_fail("the renderer is not OpenGL".into());
                };
                // SAFETY: caller keeps the `Mpv` alive until teardown.
                let mpv = unsafe { &*mpv };
                let gl =
                    unsafe { glow::Context::from_loader_function_cstr(|s| get_proc_address(s)) };
                match Video::new(gl, mpv, *get_proc_address, weak.clone()) {
                    Some(v) => {
                        video = Some(v);
                        on_ready();
                    }
                    None => on_fail("libmpv render context failed".into()),
                }
            }
            RenderingState::BeforeRendering => {
                let (Some(v), Some(w)) = (video.as_mut(), weak.upgrade()) else {
                    return;
                };
                let api = w.global::<PlayerApi>();
                if !api.get_active() {
                    return;
                }
                let scale = w.window().scale_factor();
                let px = |l: f32| ((l * scale).round() as i32).max(2);
                let used = (px(api.get_video_w()), px(api.get_video_h()));
                if let Some(img) = v.frame(used) {
                    api.set_frame(img);
                }
            }
            RenderingState::RenderingTeardown => {
                if let Some(v) = video.take() {
                    v.release();
                }
            }
            _ => {}
        })
}
