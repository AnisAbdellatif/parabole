use crate::model::{MediaKind, ProfileId, Season, SourceId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Sel {
    Favorites,
    Recent,
    Continue,
    Group(i64),
    Category(SourceId, String),
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Persisted {
    pub profile: Option<ProfileId>,
    pub tab: MediaKind,
    pub source: Option<SourceId>,
    pub sel: Option<Sel>,
    pub sidebar_width: f32,
    pub player_dock_bottom: bool,
}

impl Default for Persisted {
    fn default() -> Self {
        Self {
            profile: None,
            tab: MediaKind::Live,
            source: None,
            sel: None,
            sidebar_width: 260.0,
            player_dock_bottom: false,
        }
    }
}

#[derive(Default)]
pub struct SearchState {
    pub text: String,
    pub applied: String,
    pub edited_at: Option<Instant>,
    pub filter: super::search_filter::SearchFilter,
}

#[derive(Default)]
pub struct EditState {
    pub on: bool,
    pub filter: String,
    pub confirm_group: Option<i64>,
}

pub struct SeriesState {
    pub source: SourceId,
    pub key: String,
    pub title: String,
    pub loading: bool,
    pub seasons: Vec<Season>,
    pub season: usize,
    /// Episode to start as soon as the seasons arrive (Continue watching).
    pub autoplay: Option<String>,
    /// Watched fraction by history key (`ep:<id>`), refreshed periodically.
    pub progress: HashMap<String, f32>,
    pub progress_at: Option<Instant>,
}

pub struct Toast {
    pub text: String,
    pub born: Instant,
}
