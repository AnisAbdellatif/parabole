use super::browse::stream_row;
use super::{Db, now};
use crate::model::*;
use anyhow::Result;
use rusqlite::Row;
use std::collections::HashMap;

const QUALITY_COLS: &str =
    "q.width, q.height, q.fps, q.video_codec, q.bitrate, q.startup_ms, q.fails";

/// Maps `QUALITY_COLS` starting at column `at`.
fn quality_at(r: &Row, at: usize) -> rusqlite::Result<Quality> {
    Ok(Quality {
        width: r.get(at)?,
        height: r.get(at + 1)?,
        fps: r.get(at + 2)?,
        video_codec: r.get(at + 3)?,
        bitrate: r.get(at + 4)?,
        startup_ms: r.get(at + 5)?,
        fails: r.get(at + 6)?,
    })
}

impl Db {
    /// Stores a successful measurement and clears the failure count.
    pub fn save_quality(&self, source: SourceId, key: &str, q: &Quality) -> Result<()> {
        self.conn.execute(
            "INSERT INTO stream_quality(source_id, stream_key, width, height, fps, video_codec,
               bitrate, startup_ms, fails, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0, ?9)
             ON CONFLICT(source_id, stream_key) DO UPDATE SET width = excluded.width,
               height = excluded.height, fps = excluded.fps, video_codec = excluded.video_codec,
               bitrate = excluded.bitrate, startup_ms = excluded.startup_ms, fails = 0,
               updated_at = excluded.updated_at",
            (
                source,
                key,
                q.width,
                q.height,
                q.fps,
                q.video_codec.as_deref(),
                q.bitrate,
                q.startup_ms,
                now(),
            ),
        )?;
        Ok(())
    }

    pub fn record_failure(&self, source: SourceId, key: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO stream_quality(source_id, stream_key, fails, updated_at) VALUES (?1, ?2, 1, ?3)
             ON CONFLICT(source_id, stream_key) DO UPDATE SET fails = fails + 1,
               updated_at = excluded.updated_at",
            (source, key, now()),
        )?;
        Ok(())
    }

    /// Quality per stream id; ids without a measurement are absent.
    pub fn quality_by_stream(&self, ids: &[i64]) -> Result<HashMap<i64, Quality>> {
        let mut out = HashMap::new();
        // Stay well under SQLite's bound-parameter limit.
        for chunk in ids.chunks(500) {
            let marks = vec!["?"; chunk.len()].join(",");
            let mut stmt = self.conn.prepare(&format!(
                "SELECT s.id, {QUALITY_COLS} FROM streams s
                 JOIN stream_quality q ON q.source_id = s.source_id AND q.stream_key = s.ext_key
                 WHERE s.id IN ({marks})"
            ))?;
            let rows = stmt.query_map(rusqlite::params_from_iter(chunk), |r| {
                Ok((r.get(0)?, quality_at(r, 1)?))
            })?;
            for row in rows {
                let (id, q) = row?;
                out.insert(id, q);
            }
        }
        Ok(out)
    }

    /// Columns 0..=10 match `browse::STREAM_SELECT` so `stream_row` maps them.
    /// Visible live streams of the profile in source then provider order, with their quality.
    pub fn live_variants(
        &self,
        profile: ProfileId,
        source: Option<SourceId>,
    ) -> Result<Vec<(StreamRow, Option<Quality>)>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT s.id, s.source_id, s.ext_key, s.name, s.num, s.logo,
               COALESCE(sp.fav_position IS NOT NULL, 0), COALESCE(sp.hidden, 0), 0,
               h.position_s, h.duration_s, q.source_id IS NOT NULL, {QUALITY_COLS}
             FROM streams s
             LEFT JOIN history h ON h.profile_id = ?1 AND h.source_id = s.source_id
               AND h.media = s.media AND h.stream_key = s.ext_key
             LEFT JOIN stream_prefs sp ON sp.profile_id = ?1 AND sp.source_id = s.source_id
               AND sp.media = s.media AND sp.stream_key = s.ext_key
             JOIN profile_sources ps ON ps.profile_id = ?1 AND ps.source_id = s.source_id
             LEFT JOIN category_prefs cp ON cp.profile_id = ?1 AND cp.source_id = s.source_id
               AND cp.media = s.media AND cp.cat_key = s.category_key
             LEFT JOIN stream_quality q ON q.source_id = s.source_id AND q.stream_key = s.ext_key
             WHERE s.media = 'live' AND (?2 IS NULL OR s.source_id = ?2)
               AND COALESCE(sp.hidden, 0) = 0 AND COALESCE(cp.hidden, 0) = 0
             ORDER BY ps.position, s.provider_pos"
        ))?;
        let to_row = stream_row(MediaKind::Live);
        let rows = stmt.query_map((profile, source), |r| {
            let q = if r.get(11)? {
                Some(quality_at(r, 12)?)
            } else {
                None
            };
            Ok((to_row(r)?, q))
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }
}
