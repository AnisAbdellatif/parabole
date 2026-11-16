//! Trakt requests that failed while offline and wait to be sent again.
use super::{Db, now};
use crate::model::ProfileId;
use anyhow::Result;

impl Db {
    pub fn trakt_enqueue(&self, profile: ProfileId, path: &str, body: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO trakt_queue(profile_id, path, body, created_at) VALUES (?1, ?2, ?3, ?4)",
            (profile, path, body, now()),
        )?;
        Ok(())
    }

    /// `(id, path, body)`, oldest first.
    pub fn trakt_queued(&self, profile: ProfileId) -> Result<Vec<(i64, String, String)>> {
        let mut st = self
            .conn
            .prepare("SELECT id, path, body FROM trakt_queue WHERE profile_id = ?1 ORDER BY id")?;
        let rows = st.query_map([profile], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    pub fn trakt_dequeue(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM trakt_queue WHERE id = ?1", [id])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queue_round_trip() {
        let db = Db::open_in_memory().unwrap();
        let p = db.ensure_default_profile().unwrap();
        db.trakt_enqueue(p, "/sync/history", "{\"a\":1}").unwrap();
        db.trakt_enqueue(p, "/sync/history/remove", "{}").unwrap();
        let q = db.trakt_queued(p).unwrap();
        assert_eq!(q.len(), 2);
        assert_eq!(q[0].1, "/sync/history");
        assert_eq!(q[1].2, "{}");
        db.trakt_dequeue(q[0].0).unwrap();
        let q = db.trakt_queued(p).unwrap();
        assert_eq!(q.len(), 1);
        assert_eq!(q[0].1, "/sync/history/remove");
    }
}
