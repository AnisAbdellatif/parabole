//! Watch progress: classification, "continue watching" and watched marks.
use super::browse::stream_row;
use super::{Db, now};
use crate::model::{EpisodeRef, MediaKind, ProfileId, SourceId, StreamRow};
use anyhow::Result;
use std::collections::{HashMap, HashSet};

const STARTED: f32 = 0.02;
const WATCHED: f32 = 0.92;
const SCAN_LIMIT: i64 = 1000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Watch {
    Unstarted,
    Partial,
    Watched,
}

/// Fraction of the item played, None when position or duration is unknown.
pub fn fraction(position_s: Option<f64>, duration_s: Option<f64>) -> Option<f32> {
    match (position_s, duration_s) {
        (Some(p), Some(d)) if p > 0.0 && d > 0.0 => Some((p / d).min(1.0) as f32),
        _ => None,
    }
}

pub fn classify(fraction: Option<f32>) -> Watch {
    match fraction {
        Some(f) if f >= WATCHED => Watch::Watched,
        Some(f) if f > STARTED => Watch::Partial,
        _ => Watch::Unstarted,
    }
}

impl Db {
    /// Partially watched movies, or series with an episode in progress (newest first, one per series).
    pub fn continue_watching(
        &self,
        profile: ProfileId,
        media: MediaKind,
        limit: usize,
    ) -> Result<Vec<StreamRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT s.id, s.source_id, s.ext_key, s.name, s.num, s.logo,
               COALESCE(sp.fav_position IS NOT NULL, 0), COALESCE(sp.hidden, 0), 0,
               h.position_s, h.duration_s, h.stream_key, h.title
             FROM history h
             JOIN streams s ON s.source_id = h.source_id AND s.media = h.media
               AND s.ext_key = COALESCE(h.series_key, h.stream_key)
             JOIN profile_sources ps ON ps.profile_id = ?1 AND ps.source_id = s.source_id
             LEFT JOIN stream_prefs sp ON sp.profile_id = ?1 AND sp.source_id = s.source_id
               AND sp.media = s.media AND sp.stream_key = s.ext_key
             WHERE h.profile_id = ?1 AND h.media = ?2 AND COALESCE(sp.hidden, 0) = 0
             ORDER BY h.watched_at DESC LIMIT ?3",
        )?;
        let map = stream_row(media);
        let rows = stmt.query_map((profile, media.as_str(), SCAN_LIMIT), |r| {
            let mut row = map(r)?;
            let (key, title): (String, Option<String>) = (r.get(11)?, r.get(12)?);
            if let (Some(id), Some(label)) = (key.strip_prefix("ep:"), title) {
                row.resume_episode = Some(EpisodeRef {
                    id: id.to_string(),
                    label,
                });
            }
            Ok(row)
        })?;
        let mut seen = HashSet::new();
        let mut out = Vec::new();
        for row in rows {
            let row = row?;
            if !seen.insert((row.source_id, row.key.clone())) {
                continue;
            }
            if classify(row.progress) == Watch::Partial {
                out.push(row);
            }
        }
        out.truncate(limit);
        Ok(out)
    }

    /// Progress fraction of every episode of `source` with history, by history key (`ep:<id>`).
    pub fn episode_progress(
        &self,
        profile: ProfileId,
        source: SourceId,
    ) -> Result<HashMap<String, f32>> {
        let mut stmt = self.conn.prepare(
            "SELECT stream_key, position_s, duration_s FROM history
             WHERE profile_id = ?1 AND source_id = ?2 AND media = 'series'
               AND position_s IS NOT NULL AND duration_s IS NOT NULL",
        )?;
        let rows = stmt.query_map((profile, source), |r| {
            Ok((r.get::<_, String>(0)?, r.get(1)?, r.get(2)?))
        })?;
        let mut out = HashMap::new();
        for row in rows {
            let (key, pos, dur) = row?;
            if let Some(f) = fraction(pos, dur) {
                out.insert(key, f);
            }
        }
        Ok(out)
    }

    /// Creates/refreshes the history row of an episode with the metadata Continue watching needs.
    pub fn record_episode_watch(
        &self,
        profile: ProfileId,
        source: SourceId,
        key: &str,
        title: &str,
        series_key: &str,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO history(profile_id, source_id, media, stream_key, watched_at, title, series_key)
             VALUES (?1, ?2, 'series', ?3, ?4, ?5, ?6)
             ON CONFLICT(profile_id, source_id, media, stream_key) DO UPDATE SET
               watched_at = excluded.watched_at, title = excluded.title, series_key = excluded.series_key",
            (profile, source, key, now(), title, series_key),
        )?;
        Ok(())
    }

    /// Marks an existing history row watched (position = duration) or unwatched (position = 0).
    pub fn set_watched(
        &self,
        profile: ProfileId,
        source: SourceId,
        media: MediaKind,
        key: &str,
        watched: bool,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO history(profile_id, source_id, media, stream_key, watched_at, position_s, duration_s)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1)
             ON CONFLICT(profile_id, source_id, media, stream_key) DO UPDATE SET
               duration_s = COALESCE(NULLIF(duration_s, 0), 1),
               position_s = CASE WHEN ?6 > 0 THEN COALESCE(NULLIF(duration_s, 0), 1) ELSE 0 END",
            (profile, source, media.as_str(), key, now(), watched as i64),
        )?;
        Ok(())
    }
}
