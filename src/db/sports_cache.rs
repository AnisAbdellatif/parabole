use super::Db;
use anyhow::Result;
use rusqlite::OptionalExtension;

impl Db {
    /// Cached JSON for `key` when written with `schema` and at most `max_age_s` old (use i64::MAX for any age).
    pub fn sports_cache_get(
        &self,
        key: &str,
        schema: i64,
        max_age_s: i64,
        now: i64,
    ) -> Result<Option<String>> {
        Ok(self
            .conn
            .query_row(
                "SELECT json FROM sports_cache WHERE key = ?1 AND schema = ?2 AND fetched_at >= ?3",
                (key, schema, now.saturating_sub(max_age_s)),
                |r| r.get(0),
            )
            .optional()?)
    }

    pub fn sports_cache_put(&self, key: &str, schema: i64, now: i64, json: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO sports_cache (key, schema, fetched_at, json) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(key) DO UPDATE SET schema = ?2, fetched_at = ?3, json = ?4",
            (key, schema, now, json),
        )?;
        Ok(())
    }

    /// Deletes rows fetched more than `max_age_s` ago.
    pub fn sports_cache_prune(&self, max_age_s: i64, now: i64) -> Result<()> {
        self.conn.execute(
            "DELETE FROM sports_cache WHERE fetched_at < ?1",
            [now.saturating_sub(max_age_s)],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let db = Db::open_in_memory().unwrap();
        db.sports_cache_put("k", 1, 100, "{}").unwrap();
        assert_eq!(
            db.sports_cache_get("k", 1, 10, 105).unwrap().as_deref(),
            Some("{}")
        );
        assert_eq!(db.sports_cache_get("nope", 1, 10, 105).unwrap(), None);
    }

    #[test]
    fn wrong_schema_is_none() {
        let db = Db::open_in_memory().unwrap();
        db.sports_cache_put("k", 1, 100, "{}").unwrap();
        assert_eq!(db.sports_cache_get("k", 2, 10, 105).unwrap(), None);
    }

    #[test]
    fn too_old_is_none_but_max_age_keeps() {
        let db = Db::open_in_memory().unwrap();
        db.sports_cache_put("k", 1, 100, "{}").unwrap();
        assert_eq!(db.sports_cache_get("k", 1, 10, 111).unwrap(), None);
        assert!(
            db.sports_cache_get("k", 1, i64::MAX, 111)
                .unwrap()
                .is_some()
        );
    }

    #[test]
    fn put_overwrites() {
        let db = Db::open_in_memory().unwrap();
        db.sports_cache_put("k", 1, 100, "a").unwrap();
        db.sports_cache_put("k", 1, 200, "b").unwrap();
        assert_eq!(
            db.sports_cache_get("k", 1, 10, 205).unwrap().as_deref(),
            Some("b")
        );
    }

    #[test]
    fn prune_removes_old_only() {
        let db = Db::open_in_memory().unwrap();
        db.sports_cache_put("old", 1, 100, "a").unwrap();
        db.sports_cache_put("new", 1, 900, "b").unwrap();
        db.sports_cache_prune(500, 1000).unwrap();
        assert_eq!(db.sports_cache_get("old", 1, i64::MAX, 1000).unwrap(), None);
        assert!(
            db.sports_cache_get("new", 1, i64::MAX, 1000)
                .unwrap()
                .is_some()
        );
    }
}
