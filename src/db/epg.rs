use super::Db;
use crate::model::*;
use anyhow::{Context, Result};
use std::collections::HashSet;

impl Db {
    /// Lowercased guide channel ids used by the source's live streams.
    pub fn epg_channels(&self, source: SourceId) -> Result<HashSet<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT DISTINCT channel FROM stream_epg WHERE source_id = ?1")?;
        let rows = stmt.query_map([source], |r| r.get(0))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Replaces the source's guide in one transaction and stamps `epg_refreshed_at = now`.
    pub fn replace_epg(
        &mut self,
        source: SourceId,
        programmes: &[Programme],
        now: i64,
    ) -> Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM epg_programmes WHERE source_id = ?1", [source])?;
        {
            let mut ins = tx.prepare_cached(
                "INSERT OR IGNORE INTO epg_programmes(source_id, channel, start, stop, title, descr)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )?;
            for p in programmes {
                ins.execute((source, &p.channel, p.start, p.stop, &p.title, &p.descr))
                    .context("insert programme")?;
            }
        }
        tx.execute(
            "UPDATE sources SET epg_refreshed_at = ?2 WHERE id = ?1",
            (source, now),
        )?;
        tx.commit().context("commit epg")?;
        Ok(())
    }

    /// The M3U `url-tvg` list (comma-separated) saved by the last import.
    pub fn source_epg_url(&self, source: SourceId) -> Result<Option<String>> {
        Ok(self
            .conn
            .query_row("SELECT epg_url FROM sources WHERE id = ?1", [source], |r| {
                r.get(0)
            })?)
    }

    /// Deletes programmes that ended before `before`, all sources.
    pub fn epg_prune(&self, before: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM epg_programmes WHERE stop < ?1", [before])?;
        Ok(())
    }

    /// The profile's sources whose guide is missing or older than `cutoff`.
    pub fn epg_stale_sources(&self, profile: ProfileId, cutoff: i64) -> Result<Vec<SourceId>> {
        let mut stmt = self.conn.prepare(
            "SELECT s.id FROM sources s JOIN profile_sources ps ON ps.source_id = s.id
             WHERE ps.profile_id = ?1 AND (s.epg_refreshed_at IS NULL OR s.epg_refreshed_at < ?2)
             ORDER BY ps.position, s.id",
        )?;
        let rows = stmt.query_map((profile, cutoff), |r| r.get(0))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }
}
