use super::{Command, Worker};
use crate::model::{MediaKind, PlayRequest, ProfileId, Season, SourceId};
use anyhow::Result;
use std::sync::Arc;

/// Identifies the history row of what an embedded player is playing.
#[derive(Clone, Debug, PartialEq)]
pub struct Track {
    pub profile: ProfileId,
    pub source: SourceId,
    pub media: MediaKind,
    pub key: String,
}

/// An embedded-player request: libmpv lives on the UI thread, so the worker only resolves it.
#[derive(Clone, Debug)]
pub struct EmbedPlay {
    pub req: PlayRequest,
    pub track: Option<Track>,
    /// Row id in the stream list, for channel up/down.
    pub stream_id: Option<i64>,
    /// Set for episodes started from the series view, so the next one can follow.
    pub series: Option<SeriesCtx>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SeriesCtx {
    pub key: String,
    pub title: String,
    pub seasons: Arc<Vec<Season>>,
}

impl SeriesCtx {
    pub fn play_command(
        &self,
        profile: ProfileId,
        source: SourceId,
        episode_id: &str,
    ) -> Option<Command> {
        let (season, ep) = crate::episodes::find(&self.seasons, episode_id)?;
        Some(Command::PlayEpisode {
            profile,
            source,
            episode: ep.clone(),
            title: format!("{} {}", self.title, crate::episodes::label(season, ep)),
            kind: None,
            series: Some(self.clone()),
        })
    }
}

/// History key of an episode; the prefix keeps it from colliding with series keys.
pub fn episode_key(episode_id: &str) -> String {
    format!("ep:{episode_id}")
}

impl Worker {
    pub(super) fn save_position(
        &mut self,
        t: &Track,
        position_s: f64,
        duration_s: f64,
    ) -> Result<()> {
        self.db
            .save_position(t.profile, t.source, t.media, &t.key, position_s, duration_s)
    }

    /// Fills `req.start_s` from saved progress (VOD only).
    pub(super) fn with_resume(&self, mut req: PlayRequest, track: Option<&Track>) -> PlayRequest {
        if let Some(t) = track.filter(|t| t.media != MediaKind::Live) {
            req.start_s = self
                .db
                .resume_position(t.profile, t.source, t.media, &t.key)
                .ok()
                .flatten();
        }
        req
    }

    pub(super) fn launch_request(
        &mut self,
        req: PlayRequest,
        kind: crate::player::PlayerKind,
        track: Option<Track>,
        stream_id: Option<i64>,
    ) -> Result<()> {
        self.launch(Some(&kind), &req, track, stream_id, None);
        Ok(())
    }
}
