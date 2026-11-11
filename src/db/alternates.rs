use super::Db;
use super::alt_name::normalize;
use super::search::fts_query;
use crate::model::*;
use anyhow::Result;

const NAME_CANDIDATES: usize = 500;

impl Db {
    /// Same live channel in the profile's other sources, one per source: same non-empty `epg_id`
    /// first, then same normalized name. Best source (profile order) first.
    pub fn alternates(&self, profile: ProfileId, stream_id: i64) -> Result<Vec<Alternate>> {
        let Ok((source, media, name, epg)) = self.conn.query_row(
            "SELECT source_id, media, name, epg_id FROM streams WHERE id = ?1",
            [stream_id],
            |r| {
                Ok((
                    r.get::<_, SourceId>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, Option<String>>(3)?,
                ))
            },
        ) else {
            return Ok(Vec::new());
        };
        if media != MediaKind::Live.as_str() {
            return Ok(Vec::new());
        }
        let mut found = Vec::new();
        if let Some(epg) = epg.filter(|e| !e.trim().is_empty()) {
            found.extend(self.alt_candidates(profile, source, "s.epg_id = ?3", &epg, None)?);
        }
        let target = normalize(&name);
        if let Some(q) = fts_query(&target) {
            let clause =
                "s.id IN (SELECT rowid FROM streams_fts WHERE streams_fts MATCH ?3 LIMIT 5000)";
            found.extend(self.alt_candidates(profile, source, clause, &q, Some(&target))?);
        }
        let mut seen = Vec::new();
        found.retain(|(_, a)| {
            let fresh = !seen.contains(&a.source_name);
            seen.push(a.source_name.clone());
            fresh
        });
        Ok(found.into_iter().map(|(_, a)| a).collect())
    }

    /// `exact` = normalized name the candidate must equal (None = trust the SQL clause).
    fn alt_candidates(
        &self,
        profile: ProfileId,
        source: SourceId,
        clause: &str,
        param: &str,
        exact: Option<&str>,
    ) -> Result<Vec<(SourceId, Alternate)>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT s.id, s.source_id, s.name, src.name
             FROM streams s
             JOIN profile_sources ps ON ps.profile_id = ?1 AND ps.source_id = s.source_id
             JOIN sources src ON src.id = s.source_id
             LEFT JOIN stream_prefs sp ON sp.profile_id = ?1 AND sp.source_id = s.source_id
               AND sp.media = s.media AND sp.stream_key = s.ext_key
             LEFT JOIN category_prefs cp ON cp.profile_id = ?1 AND cp.source_id = s.source_id
               AND cp.media = s.media AND cp.cat_key = s.category_key
             WHERE s.media = 'live' AND s.source_id != ?2 AND {clause}
               AND COALESCE(sp.hidden, 0) = 0 AND COALESCE(cp.hidden, 0) = 0
             ORDER BY ps.position, s.provider_pos LIMIT {NAME_CANDIDATES}"
        ))?;
        let rows = stmt.query_map((profile, source, param), |r| {
            Ok((
                r.get::<_, SourceId>(1)?,
                r.get::<_, String>(2)?,
                Alternate {
                    stream_id: r.get(0)?,
                    source_name: r.get(3)?,
                },
            ))
        })?;
        let mut out = Vec::new();
        for row in rows {
            let (src, name, alt) = row?;
            if exact.is_none_or(|t| normalize(&name) == t) {
                out.push((src, alt));
            }
        }
        Ok(out)
    }
}
