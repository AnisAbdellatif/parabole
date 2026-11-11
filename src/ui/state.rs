use crate::model::{MediaKind, ProfileId, Season, SourceId};
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Sel {
    Favorites,
    Recent,
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
}

#[derive(Default)]
pub struct EditState {
    pub on: bool,
    pub filter: String,
}

pub struct SeriesState {
    pub source: SourceId,
    pub key: String,
    pub title: String,
    pub loading: bool,
    pub seasons: Vec<Season>,
    pub season: usize,
}

pub struct Toast {
    pub text: String,
    pub born: Instant,
}
