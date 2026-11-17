use super::Db;
use crate::model::*;
use anyhow::{Context, Result};
use rusqlite::Connection;

/// Drops the source's streams from the FTS index; call while names still match the index.
pub(super) fn fts_remove_source(conn: &Connection, source: SourceId) -> Result<()> {
    conn.execute(
        "INSERT INTO streams_fts(streams_fts, rowid, name)
         SELECT 'delete', id, name FROM streams WHERE source_id = ?1",
        [source],
    )?;
    Ok(())
}

pub(super) fn fts_add_source(conn: &Connection, source: SourceId) -> Result<()> {
    conn.execute(
        "INSERT INTO streams_fts(rowid, name) SELECT id, name FROM streams WHERE source_id = ?1",
        [source],
    )?;
    Ok(())
}

impl Db {
    /// Atomically replaces the source's categories/streams. Overlay untouched.
    pub fn replace_source_data(&mut self, source: SourceId, data: &ParsedPlaylist) -> Result<()> {
        let tx = self.conn.transaction()?;
        let old_cats = super::new_categories::keys(&tx, source)?;
        tx.execute("DELETE FROM categories WHERE source_id = ?1", [source])?;
        fts_remove_source(&tx, source)?;
        tx.execute(
            "UPDATE streams SET provider_pos = -1 WHERE source_id = ?1",
            [source],
        )?;
        {
            // OR IGNORE: a provider repeating a key must not fail the whole refresh
            let mut cat = tx.prepare(
                "INSERT OR IGNORE INTO categories(source_id, media, ext_key, name, provider_pos)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
            )?;
            for (pos, c) in data.categories.iter().enumerate() {
                cat.execute((source, c.media.as_str(), &c.ext_key, &c.name, pos as i64))?;
            }
            // upsert keeps streams.id stable; the WHERE (pos = -1 = unseen) makes the first duplicate key win
            let mut st = tx.prepare(
                "INSERT INTO streams(source_id, media, ext_key, category_key, name, num, logo,
                 url, container_ext, epg_id, archive_days, headers_json, extra_json, provider_pos)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
                 ON CONFLICT(source_id, media, ext_key) DO UPDATE SET
                   category_key = excluded.category_key, name = excluded.name, num = excluded.num,
                   logo = excluded.logo, url = excluded.url, container_ext = excluded.container_ext,
                   epg_id = excluded.epg_id, archive_days = excluded.archive_days,
                   headers_json = excluded.headers_json, extra_json = excluded.extra_json,
                   provider_pos = excluded.provider_pos
                 WHERE streams.provider_pos = -1",
            )?;
            for (pos, s) in data.streams.iter().enumerate() {
                let headers = (s.headers != Headers::default())
                    .then(|| serde_json::to_string(&s.headers))
                    .transpose()?;
                st.execute(rusqlite::params![
                    source,
                    s.media.as_str(),
                    s.ext_key,
                    s.category_key,
                    s.name,
                    s.num,
                    s.logo,
                    s.url,
                    s.container_ext,
                    s.epg_id,
                    s.archive_days,
                    headers,
                    s.extra_json,
                    pos as i64
                ])?;
            }
        }
        tx.execute(
            "DELETE FROM streams WHERE source_id = ?1 AND provider_pos = -1",
            [source],
        )?;
        fts_add_source(&tx, source)?;
        super::new_categories::hide_new(&tx, source, &old_cats, data)?;
        tx.execute(
            "UPDATE stream_prefs SET
               last_name = COALESCE((SELECT s.name FROM streams s WHERE s.source_id = stream_prefs.source_id
                 AND s.media = stream_prefs.media AND s.ext_key = stream_prefs.stream_key), last_name),
               last_logo = COALESCE((SELECT s.logo FROM streams s WHERE s.source_id = stream_prefs.source_id
                 AND s.media = stream_prefs.media AND s.ext_key = stream_prefs.stream_key), last_logo)
             WHERE source_id = ?1 AND fav_position IS NOT NULL",
            [source],
        )?;
        tx.execute(
            "UPDATE sources SET epg_url = COALESCE(?2, epg_url), account_json = COALESCE(?3, account_json)
             WHERE id = ?1",
            (source, &data.epg_url, &data.account_json),
        )?;
        tx.commit().context("commit import")?;
        Ok(())
    }
}
