use super::{Db, now};
use crate::model::{MediaKind, ProfileId, SourceId};
use anyhow::Result;

const MIN_FRACTION: f64 = 0.05;
const MAX_FRACTION: f64 = 0.95;

/// Offset to resume at, or None when barely started or (almost) finished.
pub fn resume_start(position_s: f64, duration_s: f64) -> Option<f64> {
    if duration_s <= 0.0 || position_s <= 0.0 {
        return None;
    }
    let done = position_s / duration_s;
    (done > MIN_FRACTION && done < MAX_FRACTION).then_some(position_s)
}

impl Db {
    /// Upserts the history row of a VOD item with its current playback position.
    pub fn save_position(
        &self,
        profile: ProfileId,
        source: SourceId,
        media: MediaKind,
        key: &str,
        position_s: f64,
        duration_s: f64,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO history(profile_id, source_id, media, stream_key, watched_at, position_s, duration_s)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(profile_id, source_id, media, stream_key) DO UPDATE SET
               watched_at = excluded.watched_at, position_s = excluded.position_s, duration_s = excluded.duration_s",
            (profile, source, media.as_str(), key, now(), position_s, duration_s),
        )?;
        Ok(())
    }

    pub fn resume_position(
        &self,
        profile: ProfileId,
        source: SourceId,
        media: MediaKind,
        key: &str,
    ) -> Result<Option<f64>> {
        let row: Option<(Option<f64>, Option<f64>)> = self
            .conn
            .query_row(
                "SELECT position_s, duration_s FROM history
                 WHERE profile_id = ?1 AND source_id = ?2 AND media = ?3 AND stream_key = ?4",
                (profile, source, media.as_str(), key),
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .map(Some)
            .or_else(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                e => Err(e),
            })?;
        Ok(match row {
            Some((Some(p), Some(d))) => resume_start(p, d),
            _ => None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::tests::setup;

    #[test]
    fn thresholds() {
        assert_eq!(resume_start(10.0, 1000.0), None);
        assert_eq!(resume_start(51.0, 1000.0), Some(51.0));
        assert_eq!(resume_start(500.0, 1000.0), Some(500.0));
        assert_eq!(resume_start(960.0, 1000.0), None);
        assert_eq!(resume_start(5.0, 0.0), None);
    }

    #[test]
    fn save_then_resume() {
        let (db, p, s) = setup();
        assert_eq!(
            db.resume_position(p, s, MediaKind::Movie, "k").unwrap(),
            None
        );
        db.save_position(p, s, MediaKind::Movie, "k", 600.0, 3600.0)
            .unwrap();
        assert_eq!(
            db.resume_position(p, s, MediaKind::Movie, "k").unwrap(),
            Some(600.0)
        );
        db.save_position(p, s, MediaKind::Movie, "k", 3500.0, 3600.0)
            .unwrap();
        assert_eq!(
            db.resume_position(p, s, MediaKind::Movie, "k").unwrap(),
            None
        );
    }

    #[test]
    fn record_watch_keeps_saved_position() {
        let (db, p, s) = setup();
        db.save_position(p, s, MediaKind::Movie, "k", 600.0, 3600.0)
            .unwrap();
        db.record_watch(p, s, MediaKind::Movie, "k").unwrap();
        assert_eq!(
            db.resume_position(p, s, MediaKind::Movie, "k").unwrap(),
            Some(600.0)
        );
    }
}
