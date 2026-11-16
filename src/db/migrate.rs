use anyhow::{Context, Result};
use rusqlite::Connection;

/// Ordered; index + 1 is the `user_version` after applying. Append only, never edit.
pub(super) const MIGRATIONS: &[&str] = &[
    include_str!("schema.sql"),
    include_str!("migration_2.sql"),
    include_str!("migration_3.sql"),
    include_str!("migration_4.sql"),
    include_str!("migration_5.sql"),
    include_str!("migration_6.sql"),
    include_str!("migration_7.sql"),
    include_str!("migration_8.sql"),
    include_str!("migration_9.sql"),
    include_str!("migration_10.sql"),
    include_str!("migration_11.sql"),
];

pub fn run(conn: &mut Connection) -> Result<()> {
    let current: i64 = conn.pragma_query_value(None, "user_version", |r| r.get(0))?;
    for (i, sql) in MIGRATIONS.iter().enumerate().skip(current as usize) {
        let tx = conn.transaction()?;
        tx.execute_batch(sql)
            .with_context(|| format!("migration {}", i + 1))?;
        tx.pragma_update(None, "user_version", i as i64 + 1)?;
        tx.commit()?;
    }
    Ok(())
}
