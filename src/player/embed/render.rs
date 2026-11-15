//! GL helpers for the libmpv render API: proc-address bridging and the FBO texture mpv draws into.

use glow::{self, HasContext as _};
use std::ffi::{CStr, CString, c_void};

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
