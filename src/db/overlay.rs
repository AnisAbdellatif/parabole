use super::{Db, now};
use crate::model::*;
use anyhow::Result;

impl Db {
    pub fn set_category_hidden(
        &self,
        profile: ProfileId,
        source: SourceId,
        media: MediaKind,
        key: &str,
        hidden: bool,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO category_prefs(profile_id, source_id, media, cat_key, hidden) VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(profile_id, source_id, media, cat_key) DO UPDATE SET hidden = excluded.hidden",
            (profile, source, media.as_str(), key, hidden),
        )?;
        Ok(())
    }

    pub fn set_category_pinned(
        &self,
        profile: ProfileId,
        source: SourceId,
        media: MediaKind,
        key: &str,
        pinned: bool,
    ) -> Result<()> {
        if pinned {
            self.conn.execute(
                "INSERT INTO category_prefs(profile_id, source_id, media, cat_key, fav_position)
                 VALUES (?1, ?2, ?3, ?4, (SELECT COALESCE(MAX(fav_position), -1) + 1 FROM category_prefs
                                          WHERE profile_id = ?1 AND media = ?3))
                 ON CONFLICT(profile_id, source_id, media, cat_key)
                 DO UPDATE SET fav_position = COALESCE(category_prefs.fav_position, excluded.fav_position)",
                (profile, source, media.as_str(), key),
            )?;
        } else {
            self.conn.execute(
                "UPDATE category_prefs SET fav_position = NULL
                 WHERE profile_id = ?1 AND source_id = ?2 AND media = ?3 AND cat_key = ?4",
                (profile, source, media.as_str(), key),
            )?;
        }
        Ok(())
    }

    /// `keys` = full new order of the categories of (source, media).
    pub fn reorder_categories(
        &self,
        profile: ProfileId,
        source: SourceId,
        media: MediaKind,
        keys: &[String],
    ) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO category_prefs(profile_id, source_id, media, cat_key, position) VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(profile_id, source_id, media, cat_key) DO UPDATE SET position = excluded.position",
            )?;
            for (i, key) in keys.iter().enumerate() {
                stmt.execute((profile, source, media.as_str(), key, i as i64))?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    /// Clears custom order so provider order applies again.
    pub fn reset_category_order(
        &self,
        profile: ProfileId,
        source: SourceId,
        media: MediaKind,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE category_prefs SET position = NULL WHERE profile_id = ?1 AND source_id = ?2 AND media = ?3",
            (profile, source, media.as_str()),
        )?;
        Ok(())
    }

    pub fn set_stream_hidden(
        &self,
        profile: ProfileId,
        source: SourceId,
        media: MediaKind,
        key: &str,
        hidden: bool,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO stream_prefs(profile_id, source_id, media, stream_key, hidden) VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(profile_id, source_id, media, stream_key) DO UPDATE SET hidden = excluded.hidden",
            (profile, source, media.as_str(), key, hidden),
        )?;
        Ok(())
    }

    /// Adding appends at the end of the favorites list.
    pub fn set_favorite(
        &self,
        profile: ProfileId,
        source: SourceId,
        media: MediaKind,
        key: &str,
        favorite: bool,
    ) -> Result<()> {
        if favorite {
            self.conn.execute(
                "INSERT INTO stream_prefs(profile_id, source_id, media, stream_key, fav_position, last_name, last_logo)
                 SELECT ?1, ?2, ?3, ?4, (SELECT COALESCE(MAX(fav_position), -1) + 1 FROM stream_prefs
                                         WHERE profile_id = ?1 AND media = ?3),
                        (SELECT name FROM streams WHERE source_id = ?2 AND media = ?3 AND ext_key = ?4),
                        (SELECT logo FROM streams WHERE source_id = ?2 AND media = ?3 AND ext_key = ?4)
                 WHERE true
                 ON CONFLICT(profile_id, source_id, media, stream_key)
                 DO UPDATE SET fav_position = COALESCE(stream_prefs.fav_position, excluded.fav_position),
                               last_name = COALESCE(excluded.last_name, stream_prefs.last_name),
                               last_logo = COALESCE(excluded.last_logo, stream_prefs.last_logo)",
                (profile, source, media.as_str(), key),
            )?;
        } else {
            self.conn.execute(
                "UPDATE stream_prefs SET fav_position = NULL
                 WHERE profile_id = ?1 AND source_id = ?2 AND media = ?3 AND stream_key = ?4",
                (profile, source, media.as_str(), key),
            )?;
        }
        Ok(())
    }

    /// `items` = full new order of (source, key) favorites for `media`.
    pub fn reorder_favorites(
        &self,
        profile: ProfileId,
        media: MediaKind,
        items: &[(SourceId, String)],
    ) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO stream_prefs(profile_id, source_id, media, stream_key, fav_position) VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(profile_id, source_id, media, stream_key) DO UPDATE SET fav_position = excluded.fav_position",
            )?;
            for (i, (source, key)) in items.iter().enumerate() {
                stmt.execute((profile, source, media.as_str(), key, i as i64))?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn record_watch(
        &self,
        profile: ProfileId,
        source: SourceId,
        media: MediaKind,
        key: &str,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO history(profile_id, source_id, media, stream_key, watched_at) VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(profile_id, source_id, media, stream_key) DO UPDATE SET watched_at = excluded.watched_at",
            (profile, source, media.as_str(), key, now()),
        )?;
        Ok(())
    }
}
