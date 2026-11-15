//! App state held on the UI thread, plus the small persisted part.
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
}

impl Sel {
    pub fn to_id(&self) -> String {
        match self {
            Sel::Favorites => "fav".into(),
            Sel::Recent => "recent".into(),
            Sel::Continue => "continue".into(),
            Sel::Group(id) => format!("g:{id}"),
            Sel::Category(s, k) => format!("c:{s}:{k}"),
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
                let (src, key) = s.strip_prefix("c:")?.split_once(':')?;
                Some(Sel::Category(src.parse().ok()?, key.to_owned()))
            }
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Persisted {
    pub profile: Option<ProfileId>,
    pub tab: MediaKind,
    pub source: Option<SourceId>,
    pub sel: Option<Sel>,
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
    pub worker: worker::Handle,
    pub win: slint::Weak<AppWindow>,
    pub state: Persisted,
    pub state_path: PathBuf,
    pub profiles: Vec<Profile>,
    pub sources: Vec<Source>,
    pub categories: Vec<CategoryRow>,
    pub groups: Vec<StreamGroup>,
    pub streams: Rc<StreamModel>,
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
    #[expect(dead_code, reason = "phase-2 scaffold; delete once the area uses it")]
    pub browse: super::browse::BrowseArea,
    #[expect(dead_code, reason = "phase-2 scaffold; delete once the area uses it")]
    pub dialogs: super::dialogs::DialogArea,
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
        std::fs::remove_file(&p).unwrap();
    }
}
