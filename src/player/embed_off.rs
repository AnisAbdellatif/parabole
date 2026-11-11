//! Stand-in used when built without the `embedded-mpv` feature: it never initializes.

use super::state::TrackKind;
use super::{Notice, PlayerSettings, PlayerState, Stats};
use crate::model::PlayRequest;
use anyhow::{Result, bail};

pub struct EmbeddedPlayer {
    state: PlayerState,
}

impl EmbeddedPlayer {
    pub fn new(_cc: &eframe::CreationContext<'_>, _settings: &PlayerSettings) -> Result<Self> {
        bail!("built without the embedded-mpv feature")
    }
    pub fn state(&self) -> &PlayerState {
        &self.state
    }
    pub fn poll(&mut self) -> Vec<Notice> {
        Vec::new()
    }
    pub fn paint(&self, _ui: &egui::Ui, _frame: &mut eframe::Frame, _rect: egui::Rect) {}
    pub fn play(&mut self, _settings: &PlayerSettings, _req: &PlayRequest) -> Result<()> {
        bail!("built without the embedded-mpv feature")
    }
    pub fn reload(&mut self, _settings: &PlayerSettings) -> Result<()> {
        bail!("built without the embedded-mpv feature")
    }
    pub fn stop(&mut self) {}
    pub fn is_loaded(&self) -> bool {
        false
    }
    pub fn set_paused(&mut self, _paused: bool) {}
    pub fn set_volume(&mut self, _volume: f64) {}
    pub fn set_muted(&mut self, _muted: bool) {}
    pub fn seek_to(&self, _seconds: f64) {}
    pub fn select_track(&self, _kind: TrackKind, _id: Option<i64>) {}
    pub fn stats(&self) -> Stats {
        Stats::default()
    }
}
