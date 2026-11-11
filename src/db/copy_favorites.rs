use super::Db;
use crate::model::*;
use anyhow::Result;

/// (source, media, stream_key, last_name, last_logo)
type FavRow = (SourceId, String, String, Option<String>, Option<String>);

impl Db {
    /// One-time merge: favorites of `from` that `to` lacks are appended after its own, keeping
    /// their order. Only sources linked to `to` are copied (others would be invisible). Returns the count.
    pub fn copy_favorites(&self, from: ProfileId, to: ProfileId) -> Result<usize> {
        let tx = self.conn.unchecked_transaction()?;
        let items: Vec<FavRow> = {
            let mut stmt = tx.prepare(
                "SELECT sp.source_id, sp.media, sp.stream_key, sp.last_name, sp.last_logo
                 FROM stream_prefs sp
                 JOIN profile_sources ps ON ps.profile_id = ?2 AND ps.source_id = sp.source_id
                 WHERE sp.profile_id = ?1 AND sp.fav_position IS NOT NULL
                   AND NOT EXISTS (SELECT 1 FROM stream_prefs t
                     WHERE t.profile_id = ?2 AND t.source_id = sp.source_id AND t.media = sp.media
                       AND t.stream_key = sp.stream_key AND t.fav_position IS NOT NULL)
                 ORDER BY sp.media, sp.fav_position",
            )?;
            let rows = stmt.query_map((from, to), |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
            })?;
            rows.collect::<rusqlite::Result<_>>()?
        };
        for (source, media, key, name, logo) in &items {
            tx.execute(
                "INSERT INTO stream_prefs(profile_id, source_id, media, stream_key, fav_position, last_name, last_logo)
                 VALUES (?1, ?2, ?3, ?4, (SELECT COALESCE(MAX(fav_position), -1) + 1 FROM stream_prefs
                                          WHERE profile_id = ?1 AND media = ?3), ?5, ?6)
                 ON CONFLICT(profile_id, source_id, media, stream_key)
                 DO UPDATE SET fav_position = excluded.fav_position,
                               last_name = COALESCE(stream_prefs.last_name, excluded.last_name),
                               last_logo = COALESCE(stream_prefs.last_logo, excluded.last_logo)",
                (to, source, media, key, name, logo),
            )?;
        }
        tx.commit()?;
        Ok(items.len())
    }
}
