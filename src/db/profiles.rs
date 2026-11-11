use super::Db;
use crate::model::*;
use anyhow::{Context, Result};

impl Db {
    pub fn profiles(&self) -> Result<Vec<Profile>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name FROM profiles ORDER BY id")?;
        let rows = stmt.query_map([], |r| {
            Ok(Profile {
                id: r.get(0)?,
                name: r.get(1)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Returns the first profile, creating "Default" if none exist.
    pub fn ensure_default_profile(&self) -> Result<ProfileId> {
        match self.profiles()?.first() {
            Some(p) => Ok(p.id),
            None => self.create_profile("Default"),
        }
    }

    pub fn create_profile(&self, name: &str) -> Result<ProfileId> {
        self.conn
            .execute("INSERT INTO profiles(name) VALUES (?1)", [name])
            .context("create profile")?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn rename_profile(&self, id: ProfileId, name: &str) -> Result<()> {
        self.conn
            .execute("UPDATE profiles SET name = ?2 WHERE id = ?1", (id, name))
            .context("rename profile")?;
        Ok(())
    }

    /// Deletes the profile and its overlay. Sources stay (they may be shared).
    pub fn delete_profile(&self, id: ProfileId) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute("DELETE FROM settings WHERE profile_id = ?1", [id])?;
        tx.execute("DELETE FROM profiles WHERE id = ?1", [id])?;
        tx.commit()?;
        Ok(())
    }
}
