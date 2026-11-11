use super::Db;
use crate::model::*;
use anyhow::{Result, anyhow};
use rusqlite::Row;

/// Stream columns plus the overlay join; `?1` must be the profile id.
pub(super) const STREAM_SELECT: &str = "SELECT s.id, s.source_id, s.ext_key, s.name, s.num, s.logo,
    COALESCE(sp.fav_position IS NOT NULL, 0), COALESCE(sp.hidden, 0)
    FROM streams s
    LEFT JOIN stream_prefs sp ON sp.profile_id = ?1 AND sp.source_id = s.source_id
      AND sp.media = s.media AND sp.stream_key = s.ext_key";

pub(super) fn stream_row(media: MediaKind) -> impl Fn(&Row) -> rusqlite::Result<StreamRow> {
    move |r| {
        Ok(StreamRow {
            id: r.get(0)?,
            source_id: r.get(1)?,
            media,
            key: r.get(2)?,
            name: r.get(3)?,
            num: r.get(4)?,
            logo: r.get(5)?,
            favorite: r.get(6)?,
            hidden: r.get(7)?,
        })
    }
}

impl Db {
    /// `source: None` = all sources linked to the profile. Pinned first, then custom position, then provider order.
    pub fn categories(
        &self,
        profile: ProfileId,
        source: Option<SourceId>,
        media: MediaKind,
        include_hidden: bool,
    ) -> Result<Vec<CategoryRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT c.source_id, c.ext_key, c.name,
               (SELECT COUNT(*) FROM streams s
                  LEFT JOIN stream_prefs sp ON sp.profile_id = ?1 AND sp.source_id = s.source_id
                    AND sp.media = s.media AND sp.stream_key = s.ext_key
                WHERE s.source_id = c.source_id AND s.media = c.media AND s.category_key = c.ext_key
                  AND COALESCE(sp.hidden, 0) = 0),
               COALESCE(p.hidden, 0), p.fav_position IS NOT NULL
             FROM categories c
             JOIN profile_sources ps ON ps.profile_id = ?1 AND ps.source_id = c.source_id
             LEFT JOIN category_prefs p ON p.profile_id = ?1 AND p.source_id = c.source_id
               AND p.media = c.media AND p.cat_key = c.ext_key
             WHERE c.media = ?2 AND (?3 IS NULL OR c.source_id = ?3) AND (?4 OR COALESCE(p.hidden, 0) = 0)
             ORDER BY p.fav_position IS NULL, p.fav_position, ps.position,
                      p.position IS NULL, p.position, c.provider_pos",
        )?;
        let rows = stmt.query_map((profile, media.as_str(), source, include_hidden), |r| {
            Ok(CategoryRow {
                source_id: r.get(0)?,
                media,
                key: r.get(1)?,
                name: r.get(2)?,
                count: r.get(3)?,
                hidden: r.get(4)?,
                pinned: r.get(5)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    pub fn streams_in_category(
        &self,
        profile: ProfileId,
        source: SourceId,
        media: MediaKind,
        category_key: &str,
        include_hidden: bool,
    ) -> Result<Vec<StreamRow>> {
        let mut stmt = self.conn.prepare(&format!(
            "{STREAM_SELECT}
             JOIN profile_sources ps ON ps.profile_id = ?1 AND ps.source_id = s.source_id
             WHERE s.source_id = ?2 AND s.media = ?3 AND s.category_key = ?4
               AND (?5 OR COALESCE(sp.hidden, 0) = 0)
             ORDER BY s.provider_pos"
        ))?;
        let rows = stmt.query_map(
            (
                profile,
                source,
                media.as_str(),
                category_key,
                include_hidden,
            ),
            stream_row(media),
        )?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Favorites across all linked sources, by favorite position.
    pub fn favorites(&self, profile: ProfileId, media: MediaKind) -> Result<Vec<StreamRow>> {
        let mut stmt = self.conn.prepare(&format!(
            "{STREAM_SELECT}
             JOIN profile_sources ps ON ps.profile_id = ?1 AND ps.source_id = s.source_id
             WHERE s.media = ?2 AND sp.fav_position IS NOT NULL
             ORDER BY sp.fav_position"
        ))?;
        let rows = stmt.query_map((profile, media.as_str()), stream_row(media))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Most recent first.
    pub fn recent(
        &self,
        profile: ProfileId,
        media: MediaKind,
        limit: usize,
    ) -> Result<Vec<StreamRow>> {
        let mut stmt = self.conn.prepare(&format!(
            "{STREAM_SELECT}
             JOIN profile_sources ps ON ps.profile_id = ?1 AND ps.source_id = s.source_id
             JOIN history h ON h.profile_id = ?1 AND h.source_id = s.source_id
               AND h.media = s.media AND h.stream_key = s.ext_key
             WHERE s.media = ?2
             ORDER BY h.watched_at DESC LIMIT ?3"
        ))?;
        let rows = stmt.query_map((profile, media.as_str(), limit as i64), stream_row(media))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    pub fn play_info(&self, stream_id: i64) -> Result<PlayInfo> {
        let (media, headers, mut info) = self
            .conn
            .query_row(
                "SELECT media, headers_json, source_id, ext_key, name, url, container_ext
                 FROM streams WHERE id = ?1",
                [stream_id],
                |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, Option<String>>(1)?,
                        PlayInfo {
                            source_id: r.get(2)?,
                            media: MediaKind::Live,
                            key: r.get(3)?,
                            name: r.get(4)?,
                            url: r.get(5)?,
                            container_ext: r.get(6)?,
                            headers: Headers::default(),
                        },
                    ))
                },
            )
            .map_err(|e| anyhow!("stream {stream_id}: {e}"))?;
        info.media =
            MediaKind::parse(&media).ok_or_else(|| anyhow!("stream {stream_id}: bad media"))?;
        if let Some(h) = headers {
            info.headers = serde_json::from_str(&h)?;
        }
        Ok(info)
    }
}
