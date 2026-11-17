//! App state held on the UI thread, plus the small persisted part.
use super::appearance::Appearance;
use super::images::Images;
use super::stream_model::StreamModel;
use super::{AppWindow, ToastItem};
use crate::db::Db;
use crate::model::{CategoryRow, MediaKind, Profile, ProfileId, Source, SourceId, StreamGroup};
use crate::worker;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Instant;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Sel {
    Favorites,
    Recent,
    Continue,
    Group(i64),
    Category(SourceId, String),
    /// Smart tab quality bucket (`smart::Bucket::index`).
    Quality(u8),
}

impl Sel {
    pub fn to_id(&self) -> String {
        match self {
            Sel::Favorites => "fav".into(),
            Sel::Recent => "recent".into(),
            Sel::Continue => "continue".into(),
            Sel::Group(id) => format!("g:{id}"),
            Sel::Category(s, k) => format!("c:{s}:{k}"),
            Sel::Quality(b) => format!("q:{b}"),
        }
    }

    pub fn from_id(s: &str) -> Option<Sel> {
        match s {
            "fav" => Some(Sel::Favorites),
            "recent" => Some(Sel::Recent),
            "continue" => Some(Sel::Continue),
            _ => {
                if let Some(id) = s.strip_prefix("g:") {
                    return id.parse().ok().map(Sel::Group);
                }
                if let Some(b) = s.strip_prefix("q:") {
                    return b.parse().ok().map(Sel::Quality);
                }
                let (src, key) = s.strip_prefix("c:")?.split_once(':')?;
                Some(Sel::Category(src.parse().ok()?, key.to_owned()))
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Persisted {
    pub profile: Option<ProfileId>,
    pub tab: MediaKind,
    /// The Smart tab is showing; `tab` is then Live.
    pub smart: bool,
    pub source: Option<SourceId>,
    pub sel: Option<Sel>,
    pub sidebar_width: f32,
    pub appearance: Appearance,
    pub player_dock_bottom: bool,
    pub player_pane_w: f32,
    pub player_pane_h: f32,
}

impl Default for Persisted {
    fn default() -> Self {
        Self {
            profile: None,
            tab: MediaKind::default(),
            smart: false,
            source: None,
            sel: None,
            sidebar_width: 260.0,
            appearance: Appearance::default(),
            player_dock_bottom: false,
            player_pane_w: 420.0,
            player_pane_h: 300.0,
        }
    }
}

impl Persisted {
    /// Missing or unreadable file gives the default.
    pub fn load(path: &Path) -> Self {
        std::fs::read(path)
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_vec(self).context("encode ui state")?;
        std::fs::write(path, json).context("write ui state")
    }
}

pub struct SlintApp {
    pub db: Db,
    /// All UI writes go through this, never `db` (see `writer.rs`).
    pub writer: super::writer::Writer,
    pub worker: worker::Handle,
    pub win: slint::Weak<AppWindow>,
    pub state: Persisted,
    pub state_path: PathBuf,
    pub profiles: Vec<Profile>,
    pub sources: Vec<Source>,
    pub categories: Vec<CategoryRow>,
    pub groups: Vec<StreamGroup>,
    /// Smart tab channels; empty outside it.
    pub smart: Vec<crate::smart::Channel>,
    pub scan_status: Option<crate::worker::ScanStatus>,
    pub streams: Rc<StreamModel>,
    pub play_req: Option<i64>,
    pub playing_id: Option<i64>,
    pub search_text: String,
    pub search_applied: String,
    pub search_timer: slint::Timer,
    pub refreshing: HashSet<SourceId>,
    pub toasts: Rc<slint::VecModel<ToastItem>>,
    pub toast_born: Vec<(i32, Instant)>,
    pub toast_timer: slint::Timer,
    pub next_toast: i32,
    pub images: Rc<Images>,
    pub player: super::player::PlayerArea,
    pub browse: super::browse::BrowseArea,
    pub dialogs: super::dialogs::DialogArea,
    pub palette: super::palette::PaletteArea,
    pub sports: super::sports::SportsArea,
    pub epg: super::epg::EpgArea,
}

thread_local! {
    pub static APP: RefCell<Option<SlintApp>> = const { RefCell::new(None) };
}

/// Runs `f` on the app state; None before init (or when re-entered).
pub fn with_app<R>(f: impl FnOnce(&mut SlintApp) -> R) -> Option<R> {
    APP.with(|a| a.try_borrow_mut().ok()?.as_mut().map(f))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sel_ids_round_trip() {
        for sel in [
            Sel::Favorites,
            Sel::Recent,
            Sel::Continue,
            Sel::Group(7),
            Sel::Category(3, "a:b:c".into()),
            Sel::Quality(4),
        ] {
            assert_eq!(Sel::from_id(&sel.to_id()), Some(sel));
        }
        assert_eq!(Sel::from_id("nope"), None);
    }

    #[test]
    fn corrupt_state_file_loads_default() {
        let p = std::env::temp_dir().join(format!("slint-state-{}.json", std::process::id()));
        std::fs::write(&p, b"{not json").unwrap();
        assert!(Persisted::load(&p).sel.is_none());
        std::fs::write(&p, b"{}").unwrap();
        assert_eq!(Persisted::load(&p).sidebar_width, 260.0);
        std::fs::remove_file(&p).unwrap();
    }

    #[test]
    fn old_state_file_gets_pane_defaults() {
        let s: Persisted = serde_json::from_str(r#"{"tab":"Live"}"#).unwrap();
        assert_eq!((s.player_pane_w, s.player_pane_h), (420.0, 300.0));
        assert!(!s.player_dock_bottom);
    }
}
