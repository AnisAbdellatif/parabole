use super::{Event, Worker, player_settings};
use crate::model::{Episode, Headers, PlayRequest, ProfileId, SourceId};
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
        let req = PlayRequest {
            url: sources::resolve_url(&src, &info)?,
            title: info.name.clone(),
            headers,
        };
        if self.launch(kind, &req) {
            self.db
                .record_watch(profile, info.source_id, info.media, &info.key)?;
        }
        Ok(())
    }

    pub(super) fn play_episode(
        &mut self,
        source: SourceId,
        episode: &Episode,
        title: String,
        kind: Option<&PlayerKind>,
    ) -> Result<()> {
        let src = self.db.source(source)?;
        let req = PlayRequest {
            url: sources::episode_url(&src, &episode.id, episode.container_ext.as_deref())?,
            title,
            headers: Headers {
                user_agent: src.user_agent.clone(),
                ..Default::default()
            },
        };
        self.launch(kind, &req);
        Ok(())
    }

    pub(super) fn copy_url(&mut self, stream_id: i64) -> Result<()> {
        let info = self.db.play_info(stream_id)?;
        let src = self.db.source(info.source_id)?;
        self.emit(Event::UrlReady(sources::resolve_url(&src, &info)?));
        Ok(())
    }

    fn launch(&mut self, kind: Option<&PlayerKind>, req: &PlayRequest) -> bool {
        let settings = player_settings::load(&self.db);
        let kind = kind.unwrap_or(&settings.default).clone();
        match self.player.play(&kind, &settings, req) {
            Ok(()) => {
                self.was_playing = true;
                self.emit(Event::Playing {
                    title: req.title.clone(),
                    kind,
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
