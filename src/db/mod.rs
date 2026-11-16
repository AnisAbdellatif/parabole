//! SQLite storage. Provider tables are replaced on refresh; user overlay tables
//! (prefs, favorites, history) are keyed by natural keys and survive (PLAN.md §4).
//! Owner: db agent. Public API below is a contract used by the worker/UI.

mod account;
pub mod alt_name;
mod alternates;
mod browse;
mod bulk;
mod copy_favorites;
mod epg;
mod epg_query;
mod groups;
mod import;
mod migrate;
mod new_categories;
mod overlay;
mod profiles;
pub mod progress;
mod quality;
mod resume;
mod scan;
mod search;
mod settings;
mod sources;
mod sports_cache;
#[cfg(test)]
mod tests;
pub mod track_prefs;

use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::Path;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Db {
    conn: Connection,
}

/// Fields needed to create a source; id/last_* are ignored.
pub type NewSource = crate::model::Source;

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

impl Db {
    pub fn open(path: &Path) -> Result<Db> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).with_context(|| format!("create {}", dir.display()))?;
        }
        let conn = Connection::open(path).with_context(|| format!("open {}", path.display()))?;
        conn.execute_batch("PRAGMA journal_mode = WAL;")?;
        Self::init(conn)
    }

    pub fn open_in_memory() -> Result<Db> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(mut conn: Connection) -> Result<Db> {
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        // The quality scanner writes from its own connection.
        conn.busy_timeout(Duration::from_secs(5))?;
        migrate::run(&mut conn)?;
        Ok(Db { conn })
    }
}
