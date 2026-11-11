use super::Db;
use crate::model::*;
use anyhow::Result;
use rusqlite::Connection;
use std::collections::HashSet;

fn setting_key(source: SourceId) -> String {
    format!("hide_new:{source}")
}

pub(super) fn keys(conn: &Connection, source: SourceId) -> Result<HashSet<(String, String)>> {
    let mut stmt = conn.prepare("SELECT media, ext_key FROM categories WHERE source_id = ?1")?;
    let rows = stmt.query_map([source], |r| Ok((r.get(0)?, r.get(1)?)))?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

/// Hides categories absent from `old` for every profile that enabled the option.
/// shortcut: an empty `old` (first import) hides nothing; a source emptied by a bad refresh also skips.
pub(super) fn hide_new(
    conn: &Connection,
    source: SourceId,
    old: &HashSet<(String, String)>,
    data: &ParsedPlaylist,
) -> Result<()> {
    if old.is_empty() {
        return Ok(());
    }
    let profiles: Vec<ProfileId> = conn
        .prepare(
            "SELECT profile_id FROM settings WHERE key = ?1 AND value = '1' AND profile_id <> 0",
        )?
        .query_map([setting_key(source)], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    let mut ins = conn.prepare(
        "INSERT INTO category_prefs(profile_id, source_id, media, cat_key, hidden) VALUES (?1, ?2, ?3, ?4, 1)
         ON CONFLICT(profile_id, source_id, media, cat_key) DO NOTHING",
    )?;
    for c in &data.categories {
        if old.contains(&(c.media.as_str().to_string(), c.ext_key.clone())) {
            continue;
        }
        for p in &profiles {
            ins.execute((p, source, c.media.as_str(), &c.ext_key))?;
        }
    }
    Ok(())
}

impl Db {
    pub fn hide_new_categories(&self, profile: ProfileId, source: SourceId) -> Result<bool> {
        Ok(self
            .get_setting(Some(profile), &setting_key(source))?
            .as_deref()
            == Some("1"))
    }

    pub fn set_hide_new_categories(
        &self,
        profile: ProfileId,
        source: SourceId,
        on: bool,
    ) -> Result<()> {
        self.set_setting(
            Some(profile),
            &setting_key(source),
            if on { "1" } else { "0" },
        )
    }
}
