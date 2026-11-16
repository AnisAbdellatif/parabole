use super::Db;
use crate::model::*;
use anyhow::{Context, Result};
use std::collections::{HashMap, HashSet};

impl Db {
    /// Every live stream, as matching input.
    pub fn epg_stream_keys(&self) -> Result<Vec<StreamGuideKey>> {
        let mut stmt = self.conn.prepare(
            "SELECT source_id, ext_key, epg_id, name FROM streams WHERE media = 'live'
             ORDER BY source_id, id",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(StreamGuideKey {
                source_id: r.get(0)?,
                stream_key: r.get(1)?,
                epg_id: r.get(2)?,
                name: r.get(3)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// (guide, channel, display names) of enabled guides' channels that have programmes.
    pub fn epg_candidates(&self) -> Result<Vec<(GuideId, String, Vec<String>)>> {
        let mut stmt = self.conn.prepare(
            "SELECT c.guide_id, c.channel, c.names
             FROM epg_guide_channels c JOIN epg_guides g ON g.id = c.guide_id
             WHERE g.enabled = 1 AND c.programmes > 0
             ORDER BY g.position, g.id, c.channel",
        )?;
        let rows = stmt.query_map([], |r| {
            let names: String = r.get(2)?;
            Ok((
                r.get(0)?,
                r.get(1)?,
                names.split('\n').map(str::to_owned).collect(),
            ))
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Replaces all automatic links in one transaction; manual links win.
    pub fn replace_auto_links(&mut self, links: &[EpgLink]) -> Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM epg_links WHERE manual = 0", [])?;
        {
            let mut ins = tx.prepare_cached(
                "INSERT OR IGNORE INTO epg_links(source_id, stream_key, guide_id, channel)
                 VALUES (?1, ?2, ?3, ?4)",
            )?;
            for l in links {
                ins.execute((l.source_id, &l.stream_key, l.guide_id, &l.channel))
                    .context("insert link")?;
            }
        }
        tx.commit().context("commit links")?;
        Ok(())
    }

    pub fn set_manual_link(
        &self,
        source: SourceId,
        stream_key: &str,
        guide_id: GuideId,
        channel: &str,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO epg_links(source_id, stream_key, guide_id, channel, manual)
             VALUES (?1, ?2, ?3, ?4, 1)
             ON CONFLICT(source_id, stream_key)
             DO UPDATE SET guide_id = ?3, channel = ?4, manual = 1",
            (source, stream_key, guide_id, channel),
        )?;
        Ok(())
    }

    pub fn clear_manual_link(&self, source: SourceId, stream_key: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM epg_links WHERE source_id = ?1 AND stream_key = ?2 AND manual = 1",
            (source, stream_key),
        )?;
        Ok(())
    }

    pub fn manual_link(
        &self,
        source: SourceId,
        stream_key: &str,
    ) -> Result<Option<(GuideId, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT guide_id, channel FROM epg_links
             WHERE source_id = ?1 AND stream_key = ?2 AND manual = 1",
        )?;
        let mut rows = stmt.query_map((source, stream_key), |r| Ok((r.get(0)?, r.get(1)?)))?;
        Ok(rows.next().transpose()?)
    }

    /// Channels of `guide` that streams are linked to by hand (kept even when matching would drop them).
    pub fn manual_channels(&self, guide: GuideId) -> Result<HashSet<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT channel FROM epg_links WHERE guide_id = ?1 AND manual = 1")?;
        let rows = stmt.query_map([guide], |r| r.get(0))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// (guide id, guide name, channel, first display name) of enabled guides matching `query`.
    pub fn search_guide_channels(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<(GuideId, String, String, String)>> {
        let like = format!(
            "%{}%",
            query
                .replace('\\', "\\\\")
                .replace('%', "\\%")
                .replace('_', "\\_")
        );
        let mut stmt = self.conn.prepare(
            "SELECT g.id, g.name, c.channel, c.names
             FROM epg_guide_channels c JOIN epg_guides g ON g.id = c.guide_id
             WHERE g.enabled = 1 AND (c.names LIKE ?1 ESCAPE '\\' OR c.channel LIKE ?1 ESCAPE '\\')
             ORDER BY g.position, g.id, c.channel LIMIT ?2",
        )?;
        let rows = stmt.query_map((like, limit as i64), |r| {
            let names: String = r.get(3)?;
            let first = names.split('\n').next().unwrap_or_default().to_owned();
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, first))
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Linked live streams per guide.
    pub fn epg_coverage(&self) -> Result<HashMap<GuideId, usize>> {
        let mut stmt = self
            .conn
            .prepare("SELECT guide_id, count(*) FROM stream_epg GROUP BY guide_id")?;
        let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get::<_, i64>(1)? as usize)))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }
}
