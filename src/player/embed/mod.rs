//! In-app player: libmpv with the OpenGL render API, drawn through eframe's glow context.
//! libmpv and its GL state live on the UI thread; the worker never touches this.

mod commands;
mod events;
mod render;
mod stats;

use super::{Notice, PlayerSettings, PlayerState, live};
use anyhow::{Context, Result};
use libmpv2::Mpv;
use render::VideoOutput;

pub struct EmbeddedPlayer {
    // Field order matters: the video output must drop before the Mpv it renders from.
    video: VideoOutput,
    mpv: Box<Mpv>,
    state: PlayerState,
    last: Option<crate::model::PlayRequest>,
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
        let mut mpv = Box::new(
            Mpv::with_initializer(|init| {
                for (k, v) in INIT_OPTIONS {
                    init.set_option(k, v)?;
                }
                for arg in &settings.mpv_args {
                    if let Some((k, v)) = split_mpv_arg(arg) {
                        // best effort: an option this libmpv rejects must not disable the player
                        let _ = init.set_option(k, v);
                    }
                }
                Ok(())
            })
            .map_err(|e| anyhow::anyhow!("libmpv init failed: {e}"))?,
        );
        let ctx = cc.egui_ctx.clone();
        mpv.set_wakeup_callback(move || ctx.request_repaint());
        events::observe_all(&mpv).map_err(|e| anyhow::anyhow!("observe properties: {e}"))?;
        let video = VideoOutput::new(&mpv, &**loader, &cc.egui_ctx)
            .map_err(|e| anyhow::anyhow!("libmpv render context: {e}"))?;
        if let Some(script) = live::script_path(settings) {
            let _ = mpv.command("load-script", &[&script.to_string_lossy()]);
        }
        Ok(Self {
            video,
            mpv,
            state: PlayerState {
                volume: 100.0,
                ..Default::default()
            },
            last: None,
        })
    }

    pub fn state(&self) -> &PlayerState {
        &self.state
    }

    /// Applies pending mpv events to `state()`.
    pub fn poll(&mut self) -> Vec<Notice> {
        events::poll(&self.mpv, &mut self.state)
    }

    pub fn paint(&self, ui: &egui::Ui, frame: &mut eframe::Frame, rect: egui::Rect) {
        self.video.paint(ui, frame, rect);
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
