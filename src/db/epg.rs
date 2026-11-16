use super::Db;
use crate::model::*;
use anyhow::{Context, Result, bail};
use rusqlite::Row;

const GUIDE_COLS: &str = "id, source_id, url, name, enabled, position, refreshed_at, last_error";

fn guide(r: &Row) -> rusqlite::Result<Guide> {
    Ok(Guide {
        id: r.get(0)?,
        source_id: r.get(1)?,
        url: r.get(2)?,
        name: r.get(3)?,
        enabled: r.get(4)?,
        position: r.get(5)?,
        refreshed_at: r.get(6)?,
        last_error: r.get(7)?,
    })
}

impl Db {
    pub fn epg_guides(&self) -> Result<Vec<Guide>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {GUIDE_COLS} FROM epg_guides ORDER BY position, id"
        ))?;
        let rows = stmt.query_map([], guide)?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    pub fn epg_guide(&self, id: GuideId) -> Result<Guide> {
        Ok(self.conn.query_row(
            &format!("SELECT {GUIDE_COLS} FROM epg_guides WHERE id = ?1"),
            [id],
            guide,
        )?)
    }

    pub fn provider_guide(&self, source: SourceId) -> Result<Option<GuideId>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id FROM epg_guides WHERE source_id = ?1")?;
        let mut rows = stmt.query_map([source], |r| r.get(0))?;
        Ok(rows.next().transpose()?)
    }

    /// Adds a user guide at the end of the list; only http(s) URLs are accepted.
    pub fn add_epg_guide(&self, url: &str, name: &str) -> Result<GuideId> {
        let parsed = url::Url::parse(url).context("guide URL")?;
        if !matches!(parsed.scheme(), "http" | "https") {
            bail!("guide URL must be http or https");
        }
        if let Ok(id) =
            self.conn
                .query_row("SELECT id FROM epg_guides WHERE url = ?1", [url], |r| {
                    r.get(0)
                })
        {
            return Ok(id);
        }
        self.conn
            .execute(
                "INSERT INTO epg_guides(url, name, position)
                 VALUES (?1, ?2, (SELECT coalesce(max(position), 0) + 1 FROM epg_guides))",
                (url, name),
            )
            .context("insert guide")?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn set_epg_guide_enabled(&self, id: GuideId, enabled: bool) -> Result<()> {
        self.conn.execute(
            "UPDATE epg_guides SET enabled = ?2 WHERE id = ?1",
            (id, enabled),
        )?;
        Ok(())
    }

    /// Only user-added guides; a provider guide goes away with its source.
    pub fn delete_epg_guide(&self, id: GuideId) -> Result<()> {
        let n = self.conn.execute(
            "DELETE FROM epg_guides WHERE id = ?1 AND source_id IS NULL",
            [id],
        )?;
        if n == 0 {
            bail!("not a user-added guide");
        }
        Ok(())
    }

    /// Removes every user-added guide (provider guides stay).
    pub fn delete_added_epg_guides(&self) -> Result<usize> {
        Ok(self
            .conn
            .execute("DELETE FROM epg_guides WHERE source_id IS NULL", [])?)
    }

    /// Swaps position with the neighbour `delta` places away in list order (clamped, no-op at the ends).
    pub fn move_epg_guide(&self, id: GuideId, delta: i64) -> Result<()> {
        let mut ids: Vec<GuideId> = self.epg_guides()?.into_iter().map(|g| g.id).collect();
        let Some(from) = ids.iter().position(|&g| g == id) else {
            bail!("no such guide");
        };
        let to = (from as i64 + delta).clamp(0, ids.len() as i64 - 1) as usize;
        let g = ids.remove(from);
        ids.insert(to, g);
        let tx = self.conn.unchecked_transaction()?;
        for (pos, gid) in ids.iter().enumerate() {
            tx.execute(
                "UPDATE epg_guides SET position = ?2 WHERE id = ?1",
                (gid, pos as i64),
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    /// Enabled guides never refreshed or refreshed before `cutoff`, by position.
    pub fn stale_epg_guides(&self, cutoff: i64) -> Result<Vec<GuideId>> {
        let mut stmt = self.conn.prepare(
            "SELECT id FROM epg_guides
             WHERE enabled = 1 AND (refreshed_at IS NULL OR refreshed_at < ?1)
             ORDER BY position, id",
        )?;
        let rows = stmt.query_map([cutoff], |r| r.get(0))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Replaces the guide's channels and programmes in one transaction and stamps it refreshed.
    pub fn replace_guide(
        &mut self,
        id: GuideId,
        channels: &[GuideChannel],
        programmes: &[Programme],
        now: i64,
    ) -> Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM epg_guide_channels WHERE guide_id = ?1", [id])?;
        tx.execute("DELETE FROM epg_programmes WHERE guide_id = ?1", [id])?;
        {
            let mut ins = tx.prepare_cached(
                "INSERT OR IGNORE INTO epg_programmes(guide_id, channel, start, stop, title, descr)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )?;
            for p in programmes {
                ins.execute((id, &p.channel, p.start, p.stop, &p.title, &p.descr))
                    .context("insert programme")?;
            }
            let mut ch = tx.prepare_cached(
                "INSERT OR REPLACE INTO epg_guide_channels(guide_id, channel, names, programmes)
                 VALUES (?1, ?2, ?3, (SELECT count(*) FROM epg_programmes WHERE guide_id = ?1 AND channel = ?2))",
            )?;
            for c in channels {
                ch.execute((id, &c.id, c.names.join("\n")))
                    .context("insert guide channel")?;
            }
        }
        tx.execute(
            "UPDATE epg_guides SET refreshed_at = ?2, last_error = NULL WHERE id = ?1",
            (id, now),
        )?;
        tx.commit().context("commit guide")?;
        Ok(())
    }

    /// Stamps `refreshed_at` too, so a failing guide is not retried at every start.
    pub fn set_epg_guide_error(&self, id: GuideId, err: &str, now: i64) -> Result<()> {
        self.conn.execute(
            "UPDATE epg_guides SET last_error = ?2, refreshed_at = ?3 WHERE id = ?1",
            (id, err, now),
        )?;
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

    /// Deletes programmes that ended before `before`, all guides.
    pub fn epg_prune(&self, before: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM epg_programmes WHERE stop < ?1", [before])?;
        Ok(())
    }
}
