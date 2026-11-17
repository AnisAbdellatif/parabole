//! Scrobbles and watch-history marks; failed history writes wait in the queue.
use super::{Trakt, ids::ids_for, now_rfc3339, store};
use crate::model::{MediaKind, ProfileId, SourceId};
use crate::trakt::scrobble::{Action, Item, history_body, scrobble_body};
use anyhow::Result;
use serde_json::Value;

/// Stopping at or past this share of the runtime counts as watched.
const WATCHED_PCT: f32 = 80.0;

/// 2xx = done, 409 = just scrobbled, 422 = nothing worth recording.
fn settled(status: u16) -> bool {
    (200..300).contains(&status) || status == 409 || status == 422
}

/// Worth sending again later.
fn transient(status: u16) -> bool {
    status == 429 || status >= 500
}

/// Dequeue after this response: anything but a retryable failure or a 401 (kept for after re-sign-in).
fn done(res: &Result<Option<u16>>) -> bool {
    matches!(res, Ok(Some(s)) if !transient(*s) && *s != 401)
}

impl Trakt {
    fn active(&self, profile: ProfileId) -> bool {
        store::scrobble_enabled(&self.db, profile) && store::tokens(&self.db, profile).is_some()
    }

    pub(super) fn scrobble(
        &mut self,
        track: &crate::worker::Track,
        series_key: Option<String>,
        episode: Option<(i64, i64)>,
        action: Action,
        progress: f32,
    ) {
        let profile = track.profile;
        if track.media == MediaKind::Live || !self.active(profile) {
            return;
        }
        let item = match (track.media, series_key, episode) {
            (MediaKind::Movie, ..) => ids_for(
                &self.db,
                &self.client,
                track.source,
                MediaKind::Movie,
                &track.key,
            )
            .map(Item::Movie),
            (MediaKind::Series, Some(key), Some((season, number))) => ids_for(
                &self.db,
                &self.client,
                track.source,
                MediaKind::Series,
                &key,
            )
            .map(|show| Item::Episode {
                show,
                season,
                number,
            }),
            _ => None,
        };
        let Some(item) = item else { return };
        let res = self.post(profile, action.path(), &scrobble_body(&item, progress));
        match res {
            Ok(Some(s)) if !transient(s) && s != 401 => {
                if settled(s) {
                    self.flush(profile);
                }
            }
            Ok(None) => {}
            _ if action == Action::Stop && progress >= WATCHED_PCT => {
                self.enqueue(
                    profile,
                    "/sync/history",
                    &history_body(&item, Some(&now_rfc3339())),
                );
            }
            _ => {}
        }
    }

    pub(super) fn history(
        &mut self,
        profile: ProfileId,
        source: SourceId,
        series_key: &str,
        season: i64,
        number: i64,
        watched: bool,
    ) {
        if !self.active(profile) {
            return;
        }
        let Some(show) = ids_for(
            &self.db,
            &self.client,
            source,
            MediaKind::Series,
            series_key,
        ) else {
            return;
        };
        let item = Item::Episode {
            show,
            season,
            number,
        };
        let (path, body) = if watched {
            ("/sync/history", history_body(&item, Some(&now_rfc3339())))
        } else {
            ("/sync/history/remove", history_body(&item, None))
        };
        match self.post(profile, path, &body) {
            Ok(Some(s)) if !transient(s) && s != 401 => {
                if settled(s) {
                    self.flush(profile);
                }
            }
            Ok(None) => {}
            _ => self.enqueue(profile, path, &body),
        }
    }

    fn enqueue(&self, profile: ProfileId, path: &str, body: &Value) {
        let _ = self.db.trakt_enqueue(profile, path, &body.to_string());
    }

    /// Sends the profile's queued requests in order; stops at the first failure.
    pub(super) fn flush(&mut self, profile: ProfileId) {
        let Ok(queued) = self.db.trakt_queued(profile) else {
            return;
        };
        for (id, path, body) in queued {
            // unreadable bodies can never succeed
            let ok = match serde_json::from_str::<Value>(&body) {
                Ok(v) => done(&self.post(profile, &path, &v)),
                Err(_) => true,
            };
            if !ok || self.db.trakt_dequeue(id).is_err() {
                return;
            }
        }
    }

    pub(super) fn flush_all(&mut self) {
        let Ok(profiles) = self.db.profiles() else {
            return;
        };
        for p in profiles {
            if store::tokens(&self.db, p.id).is_some() {
                self.flush(p.id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn done_dequeues_only_permanent_outcomes() {
        for s in [201, 409, 404, 400] {
            assert!(done(&Ok(Some(s))), "{s}");
        }
        for s in [401, 429, 503] {
            assert!(!done(&Ok(Some(s))), "{s}");
        }
        assert!(!done(&Ok(None)));
        assert!(!done(&Err(anyhow::anyhow!("net"))));
    }
}
