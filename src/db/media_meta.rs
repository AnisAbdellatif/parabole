//! Links from provider streams to online metadata (IMDb/Trakt) and the cached metadata itself.
use super::{Db, now};
use crate::model::{MediaKind, Meta, SourceId};
use anyhow::{Context, Result};
use rusqlite::OptionalExtension;

impl Db {
    /// `(meta_id, matched_at)`; `meta_id` is None when a search found no confident match.
    pub fn media_link(
        &self,
        source: SourceId,
        media: MediaKind,
        key: &str,
    ) -> Result<Option<(Option<String>, i64)>> {
        Ok(self
            .conn
            .query_row(
                "SELECT meta_id, matched_at FROM media_links
                 WHERE source_id = ?1 AND media = ?2 AND stream_key = ?3",
                (source, media.as_str(), key),
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?)
    }

    pub fn set_media_link(
        &self,
        source: SourceId,
        media: MediaKind,
        key: &str,
        meta_id: Option<&str>,
        manual: bool,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO media_links(source_id, media, stream_key, meta_id, manual, matched_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(source_id, media, stream_key) DO UPDATE SET
               meta_id = excluded.meta_id, manual = excluded.manual, matched_at = excluded.matched_at",
            (source, media.as_str(), key, meta_id, manual, now()),
        )?;
        Ok(())
    }

    /// `(meta, fetched_at)`; unreadable stored JSON counts as absent.
    pub fn media_meta(&self, meta_id: &str) -> Result<Option<(Meta, i64)>> {
        let row: Option<(String, i64)> = self
            .conn
            .query_row(
                "SELECT json, fetched_at FROM media_meta WHERE meta_id = ?1",
                [meta_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        Ok(row.and_then(|(json, at)| Some((serde_json::from_str(&json).ok()?, at))))
    }

    pub fn put_media_meta(&self, meta_id: &str, meta: &Meta) -> Result<()> {
        let json = serde_json::to_string(meta).context("serialize meta")?;
        self.conn.execute(
            "INSERT INTO media_meta(meta_id, fetched_at, json) VALUES (?1, ?2, ?3)
             ON CONFLICT(meta_id) DO UPDATE SET fetched_at = excluded.fetched_at, json = excluded.json",
            (meta_id, now(), json),
        )?;
        Ok(())
    }

    pub fn stream_name(
        &self,
        source: SourceId,
        media: MediaKind,
        key: &str,
    ) -> Result<Option<String>> {
        Ok(self
            .conn
            .query_row(
                "SELECT name FROM streams WHERE source_id = ?1 AND media = ?2 AND ext_key = ?3",
                (source, media.as_str(), key),
                |r| r.get(0),
            )
            .optional()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::tests::{playlist, setup};

    #[test]
    fn link_roundtrip_and_overwrite() {
        let (db, _, s) = setup();
        assert_eq!(db.media_link(s, MediaKind::Movie, "k").unwrap(), None);
        db.set_media_link(s, MediaKind::Movie, "k", Some("tt1"), false)
            .unwrap();
        let (id, at) = db.media_link(s, MediaKind::Movie, "k").unwrap().unwrap();
        assert_eq!(id.as_deref(), Some("tt1"));
        assert!(at > 0);
        db.set_media_link(s, MediaKind::Movie, "k", None, true)
            .unwrap();
        let (id, _) = db.media_link(s, MediaKind::Movie, "k").unwrap().unwrap();
        assert_eq!(id, None);
        assert_eq!(db.media_link(s, MediaKind::Series, "k").unwrap(), None);
    }

    #[test]
    fn meta_roundtrip_and_garbage() {
        let (db, _, _) = setup();
        assert_eq!(db.media_meta("tt1").unwrap(), None);
        let m = Meta {
            origin: "IMDb".into(),
            title: "The Matrix".into(),
            rating: Some(8.7),
            genres: vec!["Action".into()],
            ..Default::default()
        };
        db.put_media_meta("tt1", &m).unwrap();
        assert_eq!(db.media_meta("tt1").unwrap().unwrap().0, m);
        db.conn
            .execute("UPDATE media_meta SET json = 'nope'", [])
            .unwrap();
        assert_eq!(db.media_meta("tt1").unwrap(), None);
    }

    #[test]
    fn stream_name_lookup() {
        let (mut db, _, s) = setup();
        db.replace_source_data(s, &playlist(&["c"], &[("k", "c", "EN - Heat")]))
            .unwrap();
        let name = db.stream_name(s, MediaKind::Live, "k").unwrap();
        assert_eq!(name.as_deref(), Some("EN - Heat"));
        assert_eq!(db.stream_name(s, MediaKind::Movie, "k").unwrap(), None);
    }

    #[test]
    fn links_cascade_with_source() {
        let (db, _, s) = setup();
        db.set_media_link(s, MediaKind::Movie, "k", Some("tt1"), false)
            .unwrap();
        db.delete_source(s).unwrap();
        assert_eq!(db.media_link(s, MediaKind::Movie, "k").unwrap(), None);
    }
}
