//! SQLite storage. Provider tables are replaced on refresh; user overlay tables
//! (prefs, favorites, history) are keyed by natural keys and survive (PLAN.md §4).
//! Owner: db agent. Public API below is a contract used by the worker/UI.

use crate::model::*;
use anyhow::Result;
use rusqlite::Connection;
use std::path::Path;

pub struct Db {
    conn: Connection,
}

/// Fields needed to create a source; id/last_* are ignored.
pub type NewSource = Source;

impl Db {
    pub fn open(path: &Path) -> Result<Db> {
        let _ = path;
        todo!()
    }

    pub fn open_in_memory() -> Result<Db> {
        todo!()
    }

    // --- profiles ---
    pub fn profiles(&self) -> Result<Vec<Profile>> {
        todo!()
    }
    /// Returns the first profile, creating "Default" if none exist.
    pub fn ensure_default_profile(&self) -> Result<ProfileId> {
        todo!()
    }
    pub fn create_profile(&self, name: &str) -> Result<ProfileId> {
        let _ = name;
        todo!()
    }
    pub fn rename_profile(&self, id: ProfileId, name: &str) -> Result<()> {
        let _ = (id, name);
        todo!()
    }
    /// Deletes the profile and its overlay. Sources stay (they may be shared).
    pub fn delete_profile(&self, id: ProfileId) -> Result<()> {
        let _ = id;
        todo!()
    }

    // --- sources ---
    /// Inserts the source and links it to `profile`.
    pub fn add_source(&self, profile: ProfileId, src: &NewSource) -> Result<SourceId> {
        let _ = (profile, src);
        todo!()
    }
    pub fn update_source(&self, src: &Source) -> Result<()> {
        let _ = src;
        todo!()
    }
    pub fn delete_source(&self, id: SourceId) -> Result<()> {
        let _ = id;
        todo!()
    }
    pub fn source(&self, id: SourceId) -> Result<Source> {
        let _ = id;
        todo!()
    }
    /// Sources linked to the profile, in profile order.
    pub fn profile_sources(&self, profile: ProfileId) -> Result<Vec<Source>> {
        let _ = profile;
        todo!()
    }
    pub fn link_source(&self, profile: ProfileId, source: SourceId, linked: bool) -> Result<()> {
        let _ = (profile, source, linked);
        todo!()
    }
    /// Stores the refresh outcome: Ok sets last_refresh=now and clears last_error.
    pub fn set_refresh_result(&self, source: SourceId, error: Option<&str>) -> Result<()> {
        let _ = (source, error);
        todo!()
    }

    // --- import ---
    /// Atomically replaces the source's categories/streams. Overlay untouched.
    pub fn replace_source_data(&mut self, source: SourceId, data: &ParsedPlaylist) -> Result<()> {
        let _ = (source, data);
        todo!()
    }

    // --- browse ---
    /// `source: None` = all sources linked to the profile. Order: custom position, then provider order.
    pub fn categories(
        &self,
        profile: ProfileId,
        source: Option<SourceId>,
        media: MediaKind,
        include_hidden: bool,
    ) -> Result<Vec<CategoryRow>> {
        let _ = (profile, source, media, include_hidden);
        todo!()
    }
    pub fn streams_in_category(
        &self,
        profile: ProfileId,
        source: SourceId,
        media: MediaKind,
        category_key: &str,
        include_hidden: bool,
    ) -> Result<Vec<StreamRow>> {
        let _ = (profile, source, media, category_key, include_hidden);
        todo!()
    }
    /// Favorites across all linked sources, by favorite position.
    pub fn favorites(&self, profile: ProfileId, media: MediaKind) -> Result<Vec<StreamRow>> {
        let _ = (profile, media);
        todo!()
    }
    /// Most recent first.
    pub fn recent(&self, profile: ProfileId, media: MediaKind, limit: usize) -> Result<Vec<StreamRow>> {
        let _ = (profile, media, limit);
        todo!()
    }
    /// FTS prefix search over names in linked sources (`source: None` = all).
    pub fn search(
        &self,
        profile: ProfileId,
        source: Option<SourceId>,
        media: MediaKind,
        query: &str,
        limit: usize,
    ) -> Result<Vec<StreamRow>> {
        let _ = (profile, source, media, query, limit);
        todo!()
    }
    pub fn play_info(&self, stream_id: i64) -> Result<PlayInfo> {
        let _ = stream_id;
        todo!()
    }

    // --- overlay ---
    pub fn set_category_hidden(&self, profile: ProfileId, source: SourceId, media: MediaKind, key: &str, hidden: bool) -> Result<()> {
        let _ = (profile, source, media, key, hidden);
        todo!()
    }
    pub fn set_category_pinned(&self, profile: ProfileId, source: SourceId, media: MediaKind, key: &str, pinned: bool) -> Result<()> {
        let _ = (profile, source, media, key, pinned);
        todo!()
    }
    /// `keys` = full new order of the categories of (source, media).
    pub fn reorder_categories(&self, profile: ProfileId, source: SourceId, media: MediaKind, keys: &[String]) -> Result<()> {
        let _ = (profile, source, media, keys);
        todo!()
    }
    /// Clears custom order so provider order applies again.
    pub fn reset_category_order(&self, profile: ProfileId, source: SourceId, media: MediaKind) -> Result<()> {
        let _ = (profile, source, media);
        todo!()
    }
    pub fn set_stream_hidden(&self, profile: ProfileId, source: SourceId, media: MediaKind, key: &str, hidden: bool) -> Result<()> {
        let _ = (profile, source, media, key, hidden);
        todo!()
    }
    /// Adding appends at the end of the favorites list.
    pub fn set_favorite(&self, profile: ProfileId, source: SourceId, media: MediaKind, key: &str, favorite: bool) -> Result<()> {
        let _ = (profile, source, media, key, favorite);
        todo!()
    }
    /// `items` = full new order of (source, key) favorites for `media`.
    pub fn reorder_favorites(&self, profile: ProfileId, media: MediaKind, items: &[(SourceId, String)]) -> Result<()> {
        let _ = (profile, media, items);
        todo!()
    }
    pub fn record_watch(&self, profile: ProfileId, source: SourceId, media: MediaKind, key: &str) -> Result<()> {
        let _ = (profile, source, media, key);
        todo!()
    }

    // --- settings (JSON values; profile None = global) ---
    pub fn get_setting(&self, profile: Option<ProfileId>, key: &str) -> Result<Option<String>> {
        let _ = (profile, key);
        todo!()
    }
    pub fn set_setting(&self, profile: Option<ProfileId>, key: &str, value: &str) -> Result<()> {
        let _ = (profile, key, value);
        todo!()
    }
}
