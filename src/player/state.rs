//! Plain data the UI reads from the embedded player; independent of libmpv.

use serde::Deserialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TrackKind {
    Video,
    Audio,
    Sub,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct TrackInfo {
    pub id: i64,
    #[serde(rename = "type")]
    pub kind: TrackKind,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub lang: Option<String>,
    #[serde(default)]
    pub selected: bool,
}

impl TrackInfo {
    pub fn label(&self) -> String {
        match (&self.lang, &self.title) {
            (Some(l), Some(t)) => format!("{t} ({l})"),
            (Some(x), None) | (None, Some(x)) => x.clone(),
            (None, None) => format!("Track {}", self.id),
        }
    }
}

/// Latest values of the observed properties.
#[derive(Clone, Debug, Default)]
pub struct PlayerState {
    pub paused: bool,
    pub time_pos: Option<f64>,
    pub duration: Option<f64>,
    pub volume: f64,
    pub muted: bool,
    pub buffering: bool,
    pub media_title: Option<String>,
    pub tracks: Vec<TrackInfo>,
}

/// What the owner must react to.
#[derive(Debug, PartialEq, Eq)]
pub enum Notice {
    /// The file stopped by itself: end of stream, or an error.
    Ended {
        failed: bool,
    },
    Loaded,
}

impl PlayerState {
    /// Forget per-file values when a new file starts.
    pub fn reset_file(&mut self) {
        self.time_pos = None;
        self.duration = None;
        self.buffering = false;
        self.tracks.clear();
    }
}

#[cfg(any(feature = "embedded-mpv", test))]
pub fn parse_tracks(json: &str) -> Vec<TrackInfo> {
    serde_json::from_str(json).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_track_list_json() {
        let json = r#"[{"id":1,"type":"video","selected":true,"codec":"h264"},
            {"id":2,"type":"audio","lang":"eng","title":"Stereo","selected":true},
            {"id":3,"type":"sub","lang":"fra"}]"#;
        let t = parse_tracks(json);
        assert_eq!(t.len(), 3);
        assert_eq!(t[1].label(), "Stereo (eng)");
        assert_eq!(t[2].kind, TrackKind::Sub);
        assert!(!t[2].selected);
        assert_eq!(t[0].label(), "Track 1");
    }

    #[test]
    fn bad_json_is_empty() {
        assert!(parse_tracks("nope").is_empty());
    }
}
