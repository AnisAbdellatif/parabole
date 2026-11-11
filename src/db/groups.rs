use super::Db;
use super::browse::stream_row;
use crate::model::*;
use anyhow::Result;

impl Db {
    pub fn groups(&self, profile: ProfileId, media: MediaKind) -> Result<Vec<StreamGroup>> {
        let mut stmt = self.conn.prepare(
            "SELECT g.id, g.name, (SELECT COUNT(*) FROM stream_group_items i WHERE i.group_id = g.id)
             FROM stream_groups g WHERE g.profile_id = ?1 AND g.media = ?2 ORDER BY g.position, g.id",
        )?;
        let rows = stmt.query_map((profile, media.as_str()), |r| {
            Ok(StreamGroup {
                id: r.get(0)?,
                name: r.get(1)?,
                count: r.get(2)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    pub fn create_group(&self, profile: ProfileId, media: MediaKind, name: &str) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO stream_groups(profile_id, media, name, position)
             VALUES (?1, ?2, ?3, (SELECT COALESCE(MAX(position), -1) + 1 FROM stream_groups
                                  WHERE profile_id = ?1 AND media = ?2))",
            (profile, media.as_str(), name),
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn rename_group(&self, id: i64, name: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE stream_groups SET name = ?2 WHERE id = ?1",
            (id, name),
        )?;
        Ok(())
    }

    pub fn delete_group(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM stream_groups WHERE id = ?1", [id])?;
        Ok(())
    }

    /// `ids` = full new order of the groups of (profile, media).
    pub fn reorder_groups(&self, ids: &[i64]) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        {
            let mut stmt = tx.prepare("UPDATE stream_groups SET position = ?2 WHERE id = ?1")?;
            for (i, id) in ids.iter().enumerate() {
                stmt.execute((id, i as i64))?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    /// Appends at the end; adding an item already in the group changes nothing.
    pub fn add_to_group(&self, group: i64, source: SourceId, key: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO stream_group_items(group_id, source_id, stream_key, position, last_name, last_logo)
             SELECT ?1, ?2, ?3, (SELECT COALESCE(MAX(position), -1) + 1 FROM stream_group_items WHERE group_id = ?1),
                    s.name, s.logo
             FROM stream_groups g
             LEFT JOIN streams s ON s.source_id = ?2 AND s.media = g.media AND s.ext_key = ?3
             WHERE g.id = ?1
             ON CONFLICT(group_id, source_id, stream_key) DO NOTHING",
            (group, source, key),
        )?;
        Ok(())
    }

    pub fn remove_from_group(&self, group: i64, source: SourceId, key: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM stream_group_items WHERE group_id = ?1 AND source_id = ?2 AND stream_key = ?3",
            (group, source, key),
        )?;
        Ok(())
    }

    /// `items` = full new order of (source, key) of the group.
    pub fn reorder_group_items(&self, group: i64, items: &[(SourceId, String)]) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        {
            let mut stmt = tx.prepare(
                "UPDATE stream_group_items SET position = ?4
                 WHERE group_id = ?1 AND source_id = ?2 AND stream_key = ?3",
            )?;
            for (i, (source, key)) in items.iter().enumerate() {
                stmt.execute((group, source, key, i as i64))?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    /// Items across all linked sources, in group order; vanished streams come back with `missing`.
    pub fn group_streams(
        &self,
        profile: ProfileId,
        group: i64,
        media: MediaKind,
    ) -> Result<Vec<StreamRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT COALESCE(s.id, 0), i.source_id, i.stream_key,
               COALESCE(s.name, i.last_name, i.stream_key), s.num, COALESCE(s.logo, i.last_logo),
               COALESCE(sp.fav_position IS NOT NULL, 0), COALESCE(sp.hidden, 0), s.id IS NULL,
               h.position_s, h.duration_s
             FROM stream_group_items i
             JOIN stream_groups g ON g.id = i.group_id AND g.profile_id = ?1
             JOIN profile_sources ps ON ps.profile_id = ?1 AND ps.source_id = i.source_id
             LEFT JOIN streams s ON s.source_id = i.source_id AND s.media = g.media AND s.ext_key = i.stream_key
             LEFT JOIN stream_prefs sp ON sp.profile_id = ?1 AND sp.source_id = i.source_id
               AND sp.media = g.media AND sp.stream_key = i.stream_key
             LEFT JOIN history h ON h.profile_id = ?1 AND h.source_id = i.source_id
               AND h.media = g.media AND h.stream_key = i.stream_key
             WHERE i.group_id = ?2
             ORDER BY i.position",
        )?;
        let rows = stmt.query_map((profile, group), stream_row(media))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }
}
