//! Audio/subtitle language remembered per stream.
use super::Db;
use crate::model::{MediaKind, ProfileId, SourceId};
use anyhow::Result;
use rusqlite::OptionalExtension;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LangKind {
    Audio,
    Sub,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TrackLangs {
    pub audio: Option<String>,
    /// A language code, or "off".
    pub sub: Option<String>,
}

impl Db {
    pub fn track_langs(
        &self,
        profile: ProfileId,
        source: SourceId,
        media: MediaKind,
        key: &str,
    ) -> Result<TrackLangs> {
        let row = self
            .conn
            .query_row(
                "SELECT audio_lang, sub_lang FROM stream_prefs
                 WHERE profile_id = ?1 AND source_id = ?2 AND media = ?3 AND stream_key = ?4",
                (profile, source, media.as_str(), key),
                |r| {
                    Ok(TrackLangs {
                        audio: r.get(0)?,
                        sub: r.get(1)?,
                    })
                },
            )
            .optional()?;
        Ok(row.unwrap_or_default())
    }

    pub fn set_track_lang(
        &self,
        profile: ProfileId,
        source: SourceId,
        media: MediaKind,
        key: &str,
        kind: LangKind,
        lang: &str,
    ) -> Result<()> {
        let sql = match kind {
            LangKind::Audio => {
                "INSERT INTO stream_prefs(profile_id, source_id, media, stream_key, audio_lang)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(profile_id, source_id, media, stream_key) DO UPDATE SET audio_lang = excluded.audio_lang"
            }
            LangKind::Sub => {
                "INSERT INTO stream_prefs(profile_id, source_id, media, stream_key, sub_lang)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(profile_id, source_id, media, stream_key) DO UPDATE SET sub_lang = excluded.sub_lang"
            }
        };
        self.conn
            .execute(sql, (profile, source, media.as_str(), key, lang))?;
        Ok(())
    }
}
