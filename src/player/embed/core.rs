//! Toolkit-neutral half of the embedded player: the Mpv handle and its state, no video output.

use super::{INIT_OPTIONS, events, split_mpv_arg};
use crate::model::PlayRequest;
use crate::player::{Notice, PlayerSettings, PlayerState, live};
use anyhow::Result;
use libmpv2::Mpv;

pub struct MpvCore {
    pub(super) mpv: Box<Mpv>,
    pub(super) state: PlayerState,
    pub(super) last: Option<PlayRequest>,
}

impl MpvCore {
    /// `wakeup` runs on an mpv thread whenever events are pending.
    pub fn new(
        settings: &PlayerSettings,
        wakeup: impl Fn() + Send + Sync + 'static,
    ) -> Result<Self> {
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
        mpv.set_wakeup_callback(wakeup);
        events::observe_all(&mpv).map_err(|e| anyhow::anyhow!("observe properties: {e}"))?;
        if let Some(script) = live::script_path(settings) {
            let _ = mpv.command("load-script", &[&script.to_string_lossy()]);
        }
        Ok(Self {
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

    pub fn mpv(&self) -> &Mpv {
        &self.mpv
    }

    /// Applies pending mpv events to `state()`.
    pub fn poll(&mut self) -> Vec<Notice> {
        events::poll(&self.mpv, &mut self.state)
    }
}
