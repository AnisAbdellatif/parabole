use super::PlayerKind;
use crate::model::MediaKind;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// `#[serde(default)]` keeps JSON saved by older versions loading.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PlayerSettings {
    pub default: PlayerKind,
    /// Per-media overrides of `default`; None = use `default`.
    pub live_player: Option<PlayerKind>,
    pub movie_player: Option<PlayerKind>,
    pub episode_player: Option<PlayerKind>,
    /// None = auto-detect.
    pub mpv_path: Option<PathBuf>,
    pub vlc_path: Option<PathBuf>,
    /// e.g. `potplayer {url} /title={title}`; placeholders: {url} {title} {user_agent} {referrer}
    pub custom_template: String,
    /// Extra `--key=value` args appended to mpv.
    pub mpv_args: Vec<String>,
    /// Live only: per-file mpv options that cut buffering delay.
    pub live_low_latency: bool,
    /// Live only: speed up / seek back to the live edge when playback falls behind.
    pub live_catchup: bool,
}

impl Default for PlayerSettings {
    fn default() -> Self {
        Self {
            default: PlayerKind::Mpv,
            live_player: None,
            movie_player: None,
            episode_player: None,
            mpv_path: None,
            vlc_path: None,
            custom_template: String::new(),
            mpv_args: Vec::new(),
            live_low_latency: true,
            live_catchup: true,
        }
    }
}

impl PlayerSettings {
    pub fn kind_for(&self, media: MediaKind) -> PlayerKind {
        let over = match media {
            MediaKind::Live => &self.live_player,
            MediaKind::Movie => &self.movie_player,
            MediaKind::Series => &self.episode_player,
        };
        over.clone().unwrap_or_else(|| self.default.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_json_still_loads_with_defaults() {
        let old = r#"{"default":"Vlc","mpv_path":null,"vlc_path":null,"custom_template":"","mpv_args":["--x"]}"#;
        let s: PlayerSettings = serde_json::from_str(old).unwrap();
        assert_eq!(s.default, PlayerKind::Vlc);
        assert_eq!(s.mpv_args, ["--x"]);
        assert!(s.live_low_latency && s.live_catchup);
        assert_eq!(s.live_player, None);
    }

    #[test]
    fn per_media_override_beats_default() {
        let s = PlayerSettings {
            live_player: Some(PlayerKind::Embedded),
            ..Default::default()
        };
        assert_eq!(s.kind_for(MediaKind::Live), PlayerKind::Embedded);
        assert_eq!(s.kind_for(MediaKind::Movie), PlayerKind::Mpv);
        assert_eq!(s.kind_for(MediaKind::Series), PlayerKind::Mpv);
    }

    #[test]
    fn embedded_roundtrips() {
        let s = PlayerSettings {
            episode_player: Some(PlayerKind::Embedded),
            ..Default::default()
        };
        let back: PlayerSettings =
            serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
        assert_eq!(back, s);
    }
}
