//! In-app player: libmpv with the OpenGL render API, drawn through eframe's glow context.
//! libmpv and its GL state live on the UI thread; the worker never touches this.

mod commands;
mod core;
mod events;
pub(crate) mod render;
mod stats;

pub use self::core::MpvCore;
use super::PlayerSettings;
use anyhow::{Context, Result};
use render::VideoOutput;
use std::ops::{Deref, DerefMut};

pub struct EmbeddedPlayer {
    // Field order matters: the video output must drop before the Mpv (in `core`) it renders from.
    video: VideoOutput,
    core: MpvCore,
}

/// Options set once at init; headers and low-latency options are per file (see `commands`).
const INIT_OPTIONS: [(&str, &str); 7] = [
    ("vo", "libmpv"),
    ("hwdec", "auto-safe"),
    ("ytdl", "no"),
    ("input-default-bindings", "no"),
    ("osc", "no"),
    ("keep-open", "no"),
    (
        "stream-lavf-o",
        "reconnect=1,reconnect_streamed=1,reconnect_delay_max=5",
    ),
];

impl EmbeddedPlayer {
    /// Needs the GL context current (call from `App::new`).
    pub fn new(cc: &eframe::CreationContext<'_>, settings: &PlayerSettings) -> Result<Self> {
        let loader = cc
            .get_proc_address
            .as_ref()
            .context("no OpenGL proc address loader")?;
        let ctx = cc.egui_ctx.clone();
        let core = MpvCore::new(settings, move || ctx.request_repaint())?;
        let video = VideoOutput::new(core.mpv(), &**loader, &cc.egui_ctx)
            .map_err(|e| anyhow::anyhow!("libmpv render context: {e}"))?;
        Ok(Self { video, core })
    }

    pub fn paint(&self, ui: &egui::Ui, frame: &mut eframe::Frame, rect: egui::Rect) {
        self.video.paint(ui, frame, rect);
    }
}

impl Deref for EmbeddedPlayer {
    type Target = MpvCore;
    fn deref(&self) -> &MpvCore {
        &self.core
    }
}

impl DerefMut for EmbeddedPlayer {
    fn deref_mut(&mut self) -> &mut MpvCore {
        &mut self.core
    }
}

impl Drop for EmbeddedPlayer {
    fn drop(&mut self) {
        self.video.release();
    }
}

/// `--key=value` or `--flag` (= `yes`); other shapes are ignored.
fn split_mpv_arg(arg: &str) -> Option<(&str, &str)> {
    let body = arg.strip_prefix("--")?;
    Some(body.split_once('=').unwrap_or((body, "yes")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mpv_arg_splitting() {
        assert_eq!(split_mpv_arg("--volume=50"), Some(("volume", "50")));
        assert_eq!(split_mpv_arg("--mute"), Some(("mute", "yes")));
        assert_eq!(split_mpv_arg("-x"), None);
    }
}
