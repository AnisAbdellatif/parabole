use super::Db;
use crate::model::ProfileId;
use anyhow::Result;
use rusqlite::OptionalExtension;

impl Db {
    pub fn get_setting(&self, profile: Option<ProfileId>, key: &str) -> Result<Option<String>> {
        Ok(self
            .conn
            .query_row(
                "SELECT value FROM settings WHERE profile_id = ?1 AND key = ?2",
                (profile.unwrap_or(0), key),
                |r| r.get(0),
            )
            .optional()?)
    }

    pub fn set_setting(&self, profile: Option<ProfileId>, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO settings(profile_id, key, value) VALUES (?1, ?2, ?3)
             ON CONFLICT(profile_id, key) DO UPDATE SET value = excluded.value",
            (profile.unwrap_or(0), key, value),
        )?;
        Ok(())
    }
}
