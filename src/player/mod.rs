//! External player launching (mpv, VLC, custom template). Owner: player agent.

use crate::model::PlayRequest;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerKind {
    Mpv,
    Vlc,
    Custom,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlayerSettings {
    pub default: PlayerKind,
    /// None = auto-detect.
    pub mpv_path: Option<PathBuf>,
    pub vlc_path: Option<PathBuf>,
    /// e.g. `potplayer {url} /title={title}`; placeholders: {url} {title} {user_agent} {referrer}
    pub custom_template: String,
    /// Extra `--key=value` args appended to mpv.
    pub mpv_args: Vec<String>,
}

impl Default for PlayerSettings {
    fn default() -> Self {
        Self {
            default: PlayerKind::Mpv,
            mpv_path: None,
            vlc_path: None,
            custom_template: String::new(),
            mpv_args: Vec::new(),
        }
    }
}

/// Owns the currently running external player process (one stream at a time).
#[derive(Default)]
pub struct ExternalPlayer {}

impl ExternalPlayer {
    /// Validates the URL, stops/reuses the previous player, launches the new one.
    pub fn play(&mut self, kind: &PlayerKind, settings: &PlayerSettings, req: &PlayRequest) -> Result<()> {
        let _ = (kind, settings, req);
        todo!()
    }

    pub fn stop(&mut self) {
        todo!()
    }

    pub fn is_running(&mut self) -> bool {
        todo!()
    }
}

/// Locate a player binary: settings override, PATH, then platform default locations.
pub fn find_binary(kind: &PlayerKind, settings: &PlayerSettings) -> Option<PathBuf> {
    let _ = (kind, settings);
    todo!()
}
