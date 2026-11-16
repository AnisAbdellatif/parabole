//! Which live streams the background quality scanner should probe next.

use super::Db;
use crate::model::{MediaKind, ProfileId, SourceId};
use anyhow::Result;

const CONFIGURED: &str = "scan_configured";
const SINCE: &str = "scan_since";

/// The scan selection as `sel`. ?1 profile, ?2 whether the selection was configured.
const SEL: &str = "
    WITH sel(source_id, cat_key) AS (
      SELECT source_id, cat_key FROM scan_categories WHERE profile_id = ?1 AND ?2
      UNION
      SELECT c.source_id, c.ext_key FROM categories c
        JOIN profile_sources ps ON ps.profile_id = ?1 AND ps.source_id = c.source_id
        LEFT JOIN category_prefs cp ON cp.profile_id = ?1 AND cp.source_id = c.source_id
          AND cp.media = c.media AND cp.cat_key = c.ext_key
       WHERE NOT ?2 AND c.media = 'live' AND COALESCE(cp.hidden, 0) = 0
    )";

/// Visible live streams of the profile in `sel`; `q` is their quality row.
const SCAN_FROM: &str = "
    FROM streams s
    JOIN profile_sources ps ON ps.profile_id = ?1 AND ps.source_id = s.source_id
    JOIN sel ON sel.source_id = s.source_id AND sel.cat_key = s.category_key
    LEFT JOIN stream_prefs sp ON sp.profile_id = ?1 AND sp.source_id = s.source_id
      AND sp.media = s.media AND sp.stream_key = s.ext_key
    LEFT JOIN stream_quality q ON q.source_id = s.source_id AND q.stream_key = s.ext_key
    WHERE s.media = 'live' AND COALESCE(sp.hidden, 0) = 0";

impl Db {
    fn scan_configured(&self, profile: ProfileId) -> Result<bool> {
        Ok(self.get_setting(Some(profile), CONFIGURED)?.is_some())
    }

    /// The saved selection; empty both when never configured and when saved empty.
    pub fn scan_categories(&self, profile: ProfileId) -> Result<Vec<(SourceId, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT source_id, cat_key FROM scan_categories WHERE profile_id = ?1
             ORDER BY source_id, cat_key",
        )?;
        let rows = stmt.query_map([profile], |r| Ok((r.get(0)?, r.get(1)?)))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    pub fn set_scan_categories(
        &mut self,
        profile: ProfileId,
        cats: &[(SourceId, String)],
    ) -> Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute(
            "DELETE FROM scan_categories WHERE profile_id = ?1",
            [profile],
        )?;
        for (source, key) in cats {
            tx.execute(
                "INSERT OR IGNORE INTO scan_categories(profile_id, source_id, cat_key)
                 VALUES (?1, ?2, ?3)",
                (profile, source, key),
            )?;
        }
        tx.execute(
            "INSERT INTO settings(profile_id, key, value) VALUES (?1, ?2, '1')
             ON CONFLICT(profile_id, key) DO UPDATE SET value = excluded.value",
            (profile, CONFIGURED),
        )?;
        tx.commit()?;
        Ok(())
    }

    /// Saved selection, or the profile's visible live categories when never configured.
    pub fn scan_selection_or_default(&self, profile: ProfileId) -> Result<Vec<(SourceId, String)>> {
        if self.scan_configured(profile)? {
            return self.scan_categories(profile);
        }
        let cats = self.categories(profile, None, MediaKind::Live, false)?;
        Ok(cats.into_iter().map(|c| (c.source_id, c.key)).collect())
    }

    /// Unix seconds; quality rows older than this are scanned again.
    pub fn scan_since(&self, profile: ProfileId) -> Result<i64> {
        let v = self.get_setting(Some(profile), SINCE)?;
        Ok(v.and_then(|v| v.parse().ok()).unwrap_or(0))
    }

    pub fn set_scan_since(&self, profile: ProfileId, secs: i64) -> Result<()> {
        self.set_setting(Some(profile), SINCE, &secs.to_string())
    }

    /// Next stream ids to probe: never measured, or measured before `scan_since`.
    pub fn scan_queue(&self, profile: ProfileId, limit: i64) -> Result<Vec<i64>> {
        let mut stmt = self.conn.prepare(&format!(
            "{SEL} SELECT s.id {SCAN_FROM} AND (q.source_id IS NULL OR q.updated_at < ?3)
             ORDER BY ps.position, s.provider_pos LIMIT ?4"
        ))?;
        let since = self.scan_since(profile)?;
        let configured = self.scan_configured(profile)?;
        let rows = stmt.query_map((profile, configured, since, limit), |r| r.get(0))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// (probed since `scan_since`, total) over the scan selection.
    pub fn scan_counts(&self, profile: ProfileId) -> Result<(i64, i64)> {
        let sql = format!(
            "{SEL} SELECT COALESCE(SUM(q.source_id IS NOT NULL AND q.updated_at >= ?3), 0),
               COUNT(*) {SCAN_FROM}"
        );
        let since = self.scan_since(profile)?;
        let configured = self.scan_configured(profile)?;
        Ok(self
            .conn
            .query_row(&sql, (profile, configured, since), |r| {
                Ok((r.get(0)?, r.get(1)?))
            })?)
    }
}
