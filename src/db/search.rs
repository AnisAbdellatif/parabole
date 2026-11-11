use super::Db;
use super::browse::{STREAM_SELECT, stream_row};
use crate::model::*;
use anyhow::Result;

/// Quotes every token so FTS operators in user input (`OR`, `-`, `*`, `"`) are literal.
fn fts_query(input: &str) -> Option<String> {
    let tokens: Vec<String> = input
        .split_whitespace()
        .map(|t| format!("\"{}\"*", t.replace('"', "\"\"")))
        .collect();
    (!tokens.is_empty()).then(|| tokens.join(" "))
}

impl Db {
    /// FTS prefix search over names in linked sources (`source: None` = all).
    pub fn search(
        &self,
        profile: ProfileId,
        source: Option<SourceId>,
        media: MediaKind,
        query: &str,
        limit: usize,
    ) -> Result<Vec<StreamRow>> {
        let Some(q) = fts_query(query) else {
            return Ok(Vec::new());
        };
        let mut stmt = self.conn.prepare(&format!(
            "{STREAM_SELECT}
             JOIN profile_sources ps ON ps.profile_id = ?1 AND ps.source_id = s.source_id
             LEFT JOIN category_prefs cp ON cp.profile_id = ?1 AND cp.source_id = s.source_id
               AND cp.media = s.media AND cp.cat_key = s.category_key
             WHERE s.id IN (SELECT rowid FROM streams_fts WHERE streams_fts MATCH ?2)
               AND s.media = ?3 AND (?4 IS NULL OR s.source_id = ?4)
               AND COALESCE(sp.hidden, 0) = 0 AND COALESCE(cp.hidden, 0) = 0
             ORDER BY ps.position, s.provider_pos LIMIT ?5"
        ))?;
        let rows = stmt.query_map(
            (profile, q, media.as_str(), source, limit as i64),
            stream_row(media),
        )?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }
}
