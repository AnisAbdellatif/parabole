use super::{EmbedPlay, Event, SeriesCtx, Track, Worker, episode_key, player_settings};
use crate::model::{Episode, Headers, MediaKind, PlayRequest, ProfileId, SourceId};
use crate::player::PlayerKind;
use crate::sources;
use anyhow::Result;

impl Worker {
    pub(super) fn play(
        &mut self,
        profile: ProfileId,
        stream_id: i64,
        kind: Option<&PlayerKind>,
    ) -> Result<()> {
        let info = self.db.play_info(stream_id)?;
        let src = self.db.source(info.source_id)?;
        let mut headers = info.headers.clone();
        headers.user_agent = headers.user_agent.or_else(|| src.user_agent.clone());
        let track = Track {
            profile,
            source: info.source_id,
            media: info.media,
            key: info.key.clone(),
        };
        let req = PlayRequest {
            url: sources::resolve_url(&src, &info)?,
            title: info.name.clone(),
            headers,
            media: info.media,
            start_s: None,
        };
        if self.launch(kind, &req, Some(track), Some(stream_id), None) {
            self.db
                .record_watch(profile, info.source_id, info.media, &info.key)?;
        }
        Ok(())
    }

    pub(super) fn play_episode(
        &mut self,
        profile: ProfileId,
        source: SourceId,
        episode: &Episode,
        title: String,
        kind: Option<&PlayerKind>,
        series: Option<SeriesCtx>,
    ) -> Result<()> {
        let src = self.db.source(source)?;
        let req = PlayRequest {
            url: sources::episode_url(&src, &episode.id, episode.container_ext.as_deref())?,
            title,
            headers: Headers {
                user_agent: src.user_agent.clone(),
                ..Default::default()
            },
            media: MediaKind::Series,
            start_s: None,
        };
        let track = Track {
            profile,
            source,
            media: MediaKind::Series,
            key: episode_key(&episode.id),
        };
        let key = track.key.clone();
        let launched = self.launch(kind, &req, Some(track), None, series.clone());
        if let (true, Some(ctx)) = (launched, series) {
            let label = crate::episodes::find(&ctx.seasons, &episode.id).map_or_else(
                || episode.title.clone(),
                |(s, e)| crate::episodes::label(s, e),
            );
            self.db
                .record_episode_watch(profile, source, &key, &label, &ctx.key)?;
        }
        Ok(())
    }

    pub(super) fn copy_url(&mut self, stream_id: i64) -> Result<()> {
        let info = self.db.play_info(stream_id)?;
        let src = self.db.source(info.source_id)?;
        self.emit(Event::UrlReady(sources::resolve_url(&src, &info)?));
        Ok(())
    }

    pub(super) fn launch(
        &mut self,
        kind: Option<&PlayerKind>,
        req: &PlayRequest,
        track: Option<Track>,
        stream_id: Option<i64>,
        series: Option<SeriesCtx>,
    ) -> bool {
        let settings = player_settings::load(&self.db);
        let mut kind = kind
            .cloned()
            .unwrap_or_else(|| settings.kind_for(req.media));
        if kind == PlayerKind::Embedded && !cfg!(feature = "embedded-mpv") {
            kind = PlayerKind::Mpv;
        }
        let req = self.with_resume(req.clone(), track.as_ref());
        if kind == PlayerKind::Embedded {
            self.player.stop();
            self.was_playing = false;
            self.emit(Event::PlayEmbedded(Box::new(EmbedPlay {
                req,
                track,
                stream_id,
                series,
            })));
            return true;
        }
        match self.player.play(&kind, &settings, &req) {
            Ok(()) => {
                self.was_playing = true;
                self.emit(Event::Playing {
                    title: req.title.clone(),
                    kind,
                    media: req.media,
                });
                true
            }
            Err(e) => {
                self.emit(Event::PlayFailed(format!("{e:#}")));
                false
            }
        }
    }
}
