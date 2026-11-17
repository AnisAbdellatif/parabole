use super::{Db, NewSource, now};
use crate::model::*;
use anyhow::{Context, Result, anyhow};

const COLS: &str = "s.id, s.name, s.kind, s.url, s.username, s.password, s.user_agent, \
                    s.output_format, s.last_refresh, s.last_error, s.mirrors";

fn row_to_source(r: &rusqlite::Row) -> rusqlite::Result<Source> {
    let kind: String = r.get(2)?;
    Ok(Source {
        id: r.get(0)?,
        name: r.get(1)?,
        kind: SourceKind::parse(&kind).ok_or(rusqlite::Error::InvalidColumnType(
            2,
            "kind".into(),
            rusqlite::types::Type::Text,
        ))?,
        url: r.get(3)?,
        username: r.get(4)?,
        password: r.get(5)?,
        user_agent: r.get(6)?,
        output_format: r.get(7)?,
        last_refresh: r.get(8)?,
        last_error: r.get(9)?,
        mirrors: r
            .get::<_, Option<String>>(10)?
            .map(|m| m.lines().map(str::to_string).collect())
            .unwrap_or_default(),
    })
}

fn join_mirrors(mirrors: &[String]) -> Option<String> {
    (!mirrors.is_empty()).then(|| mirrors.join("\n"))
}

impl Db {
    /// Inserts the source and links it to `profile`.
    pub fn add_source(&self, profile: ProfileId, src: &NewSource) -> Result<SourceId> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute(
            "INSERT INTO sources(name, kind, url, username, password, user_agent, output_format, mirrors)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            (
                &src.name,
                src.kind.as_str(),
                &src.url,
                &src.username,
                &src.password,
                &src.user_agent,
                &src.output_format,
                join_mirrors(&src.mirrors),
            ),
        )
        .context("insert source")?;
        let id = tx.last_insert_rowid();
        tx.execute(
            "INSERT INTO epg_guides(source_id, name) VALUES (?1, ?2)",
            (id, &src.name),
        )
        .context("insert provider guide")?;
        link(&tx, profile, id)?;
        tx.commit()?;
        Ok(id)
    }

    pub fn update_source(&self, src: &Source) -> Result<()> {
        self.conn
            .execute(
                "UPDATE sources SET name=?2, kind=?3, url=?4, username=?5, password=?6,
                 user_agent=?7, output_format=?8, mirrors=?9 WHERE id=?1",
                (
                    src.id,
                    &src.name,
                    src.kind.as_str(),
                    &src.url,
                    &src.username,
                    &src.password,
                    &src.user_agent,
                    &src.output_format,
                    join_mirrors(&src.mirrors),
                ),
            )
            .context("update source")?;
        Ok(())
    }

    /// Cascades to provider rows and overlay rows of every profile.
    pub fn delete_source(&self, id: SourceId) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        super::import::fts_remove_source(&tx, id)?;
        tx.execute("DELETE FROM sources WHERE id = ?1", [id])?;
        tx.commit()?;
        Ok(())
    }

    pub fn source(&self, id: SourceId) -> Result<Source> {
        self.conn
            .query_row(
                &format!("SELECT {COLS} FROM sources s WHERE s.id = ?1"),
                [id],
                row_to_source,
            )
            .map_err(|e| anyhow!("source {id}: {e}"))
    }

    /// Sources linked to the profile, in profile order.
    pub fn profile_sources(&self, profile: ProfileId) -> Result<Vec<Source>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {COLS} FROM sources s JOIN profile_sources ps ON ps.source_id = s.id
             WHERE ps.profile_id = ?1 ORDER BY ps.position, s.id"
        ))?;
        let rows = stmt.query_map([profile], row_to_source)?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    pub fn link_source(&self, profile: ProfileId, source: SourceId, linked: bool) -> Result<()> {
        if linked {
            link(&self.conn, profile, source)?;
        } else {
            self.conn.execute(
                "DELETE FROM profile_sources WHERE profile_id = ?1 AND source_id = ?2",
                (profile, source),
            )?;
        }
        Ok(())
    }

    /// Stores the refresh outcome: Ok sets last_refresh=now and clears last_error.
    pub fn set_refresh_result(&self, source: SourceId, error: Option<&str>) -> Result<()> {
        match error {
            None => self.conn.execute(
                "UPDATE sources SET last_refresh = ?2, last_error = NULL WHERE id = ?1",
                (source, now()),
            )?,
            Some(e) => self.conn.execute(
                "UPDATE sources SET last_error = ?2 WHERE id = ?1",
                (source, e),
            )?,
        };
        Ok(())
    }
}

fn link(conn: &rusqlite::Connection, profile: ProfileId, source: SourceId) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO profile_sources(profile_id, source_id, position)
         VALUES (?1, ?2, (SELECT COALESCE(MAX(position), -1) + 1 FROM profile_sources WHERE profile_id = ?1))",
        (profile, source),
    )?;
    Ok(())
}
