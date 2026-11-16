use super::Worker;
use crate::db::Db;
use crate::model::ProfileId;
use anyhow::Result;
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64)
}

const KEY: &str = "auto_refresh_hours";
const DEFAULT_HOURS: u32 = 24;

/// 0 = off.
pub fn hours(db: &Db) -> u32 {
    db.get_setting(None, KEY)
        .ok()
        .flatten()
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(DEFAULT_HOURS)
}

pub fn set_hours(db: &Db, hours: u32) -> Result<()> {
    db.set_setting(None, KEY, &hours.to_string())
}

impl Worker {
    /// Queues the profile's stale sources and starts the first; the rest follow as each finishes.
    pub(super) fn auto_refresh(&mut self, profile: ProfileId) -> Result<()> {
        let hours = hours(&self.db);
        if hours == 0 {
            return Ok(());
        }
        let now = now();
        let cutoff = now - i64::from(hours) * 3600;
        self.auto_queue = self
            .db
            .profile_sources(profile)?
            .into_iter()
            .filter(|s| s.last_refresh.is_none_or(|t| t < cutoff))
            .map(|s| s.id)
            .collect();
        self.next_auto();
        // TODO(epg2): worker rewrite queues stale guides here
        Ok(())
    }

    pub(super) fn next_auto(&mut self) {
        // a queued source may have been deleted meanwhile: skip it
        while let Some(id) = self.auto_queue.pop_front() {
            if self.start_refresh(id).is_ok() {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hours_default_and_roundtrip() {
        let db = Db::open_in_memory().unwrap();
        assert_eq!(hours(&db), 24);
        set_hours(&db, 0).unwrap();
        assert_eq!(hours(&db), 0);
        set_hours(&db, 6).unwrap();
        assert_eq!(hours(&db), 6);
    }
}
