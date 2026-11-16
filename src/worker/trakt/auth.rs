//! Device-flow sign-in state machine, token refresh and the authed POST helper.
use super::{POST_GAP, Trakt, TraktUpdate, now_secs, store};
use crate::model::ProfileId;
use crate::trakt::auth::{self, Poll, Tokens};
use crate::trakt::client::post_json;
use anyhow::Result;
use serde_json::Value;
use std::thread;
use std::time::{Duration, Instant};

/// Refresh when the access token has less than a day left.
const REFRESH_WITHIN_S: i64 = 86400;

pub(super) struct Pending {
    profile: ProfileId,
    device_code: String,
    interval: Duration,
    next_at: Instant,
    deadline: Instant,
}

impl Trakt {
    fn failed(&self, profile: ProfileId, error: &str) {
        self.emit(TraktUpdate::Failed {
            profile,
            error: error.into(),
        });
    }

    pub(super) fn connect(&mut self, profile: ProfileId) {
        self.pending = None;
        match auth::device_code(&self.client) {
            Ok(c) => {
                let interval = Duration::from_secs(c.interval.max(1) as u64);
                let now = Instant::now();
                self.pending = Some(Pending {
                    profile,
                    device_code: c.device_code,
                    interval,
                    next_at: now + interval,
                    deadline: now + Duration::from_secs(c.expires_in.max(1) as u64),
                });
                self.emit(TraktUpdate::Code {
                    profile,
                    user_code: c.user_code,
                    url: c.verification_url,
                });
            }
            Err(e) => self.failed(profile, &format!("{e:#}")),
        }
    }

    pub(super) fn cancel_connect(&mut self) {
        if let Some(p) = self.pending.take() {
            self.emit(TraktUpdate::Disconnected { profile: p.profile });
        }
    }

    pub(super) fn disconnect(&mut self, profile: ProfileId) {
        if self.pending.as_ref().is_some_and(|p| p.profile == profile) {
            self.pending = None;
        }
        if let Some(t) = store::tokens(&self.db, profile) {
            let _ = auth::revoke(&self.client, &t.access_token);
        }
        if let Err(e) = store::clear_tokens(&self.db, profile) {
            return self.failed(profile, &format!("{e:#}"));
        }
        self.emit(TraktUpdate::Disconnected { profile });
    }

    /// Advances a pending device poll once its next check is due.
    pub(super) fn poll_pending(&mut self) {
        let Some(mut p) = self.pending.take() else {
            return;
        };
        let now = Instant::now();
        if now >= p.deadline {
            return self.failed(p.profile, "The code expired; connect again.");
        }
        if now < p.next_at {
            self.pending = Some(p);
            return;
        }
        match auth::poll(&self.client, &p.device_code) {
            Ok(Poll::Done(tokens)) => return self.finish_connect(p.profile, tokens),
            Ok(Poll::Expired) => return self.failed(p.profile, "The code expired; connect again."),
            Ok(Poll::Denied) => return self.failed(p.profile, "Access was denied on Trakt."),
            Ok(Poll::Invalid) => {
                return self.failed(p.profile, "Trakt did not accept the code; connect again.");
            }
            Ok(Poll::SlowDown) => p.interval += Duration::from_secs(5),
            // a network blip: try again at the next interval
            Ok(Poll::Pending) | Err(_) => {}
        }
        p.next_at = Instant::now() + p.interval;
        self.pending = Some(p);
    }

    fn finish_connect(&mut self, profile: ProfileId, mut tokens: Tokens) {
        tokens.username = auth::username(&self.client, &tokens.access_token).ok();
        if let Err(e) = store::save_tokens(&self.db, profile, &tokens) {
            return self.failed(profile, &format!("{e:#}"));
        }
        self.emit(TraktUpdate::Connected {
            profile,
            username: tokens.username.unwrap_or_default(),
        });
        self.flush(profile);
    }

    /// A valid access token, refreshed first when it is about to expire; None = not connected.
    pub(super) fn access_token(&mut self, profile: ProfileId) -> Result<Option<String>> {
        let Some(t) = store::tokens(&self.db, profile) else {
            return Ok(None);
        };
        if t.expires_at - now_secs() >= REFRESH_WITHIN_S {
            return Ok(Some(t.access_token));
        }
        match self.refresh(profile, &t) {
            // offline with a still-valid token: use it
            Err(_) if t.expires_at > now_secs() => Ok(Some(t.access_token)),
            r => r,
        }
    }

    /// Refresh tokens are single-use: the new pair is saved before anything else.
    fn refresh(&mut self, profile: ProfileId, old: &Tokens) -> Result<Option<String>> {
        match auth::refresh(&self.client, &old.refresh_token)? {
            Some(mut new) => {
                new.username = old.username.clone();
                store::save_tokens(&self.db, profile, &new)?;
                Ok(Some(new.access_token))
            }
            None => {
                store::clear_tokens(&self.db, profile)?;
                self.failed(
                    profile,
                    "Trakt sign-in expired; connect again in Settings → Services.",
                );
                Ok(None)
            }
        }
    }

    /// POSTs as the user at most once per `POST_GAP`, refreshing once on a 401.
    /// `Ok(None)` = not connected; `Err` = network failure.
    pub(super) fn post(
        &mut self,
        profile: ProfileId,
        path: &str,
        body: &Value,
    ) -> Result<Option<u16>> {
        let Some(mut token) = self.access_token(profile)? else {
            return Ok(None);
        };
        let mut retried = false;
        loop {
            if let Some(at) = self.last_post {
                thread::sleep(POST_GAP.saturating_sub(at.elapsed()));
            }
            let res = post_json(&self.client, path, body, Some(&token));
            self.last_post = Some(Instant::now());
            let (status, _) = res?;
            if status != 401 || retried {
                return Ok(Some(status));
            }
            retried = true;
            let Some(old) = store::tokens(&self.db, profile) else {
                return Ok(None);
            };
            match self.refresh(profile, &old)? {
                Some(t) => token = t,
                None => return Ok(None),
            }
        }
    }
}
