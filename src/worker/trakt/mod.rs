//! One thread for all user-authenticated Trakt traffic, so single-use refresh tokens never race.
mod auth;
mod ids;
mod push;
pub mod store;

use super::{Event, Repaint, Track};
use crate::db::Db;
use crate::model::{ProfileId, SourceId};
use crate::trakt::scrobble::Action;
use anyhow::{Context, Result};
use chrono::{SecondsFormat, Utc};
use reqwest::blocking::Client;
use std::path::Path;
use std::sync::mpsc::{RecvTimeoutError, Sender, channel};
use std::thread;
use std::time::{Duration, Instant};

const TICK: Duration = Duration::from_millis(500);
/// Trakt allows one user POST per second.
const POST_GAP: Duration = Duration::from_millis(1100);

#[derive(Debug)]
pub enum Job {
    Connect(ProfileId),
    CancelConnect,
    Disconnect(ProfileId),
    Scrobble {
        track: Track,
        series_key: Option<String>,
        episode: Option<(i64, i64)>,
        action: Action,
        progress: f32,
    },
    History {
        profile: ProfileId,
        source: SourceId,
        series_key: String,
        season: i64,
        number: i64,
        watched: bool,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum TraktUpdate {
    Code {
        profile: ProfileId,
        user_code: String,
        url: String,
    },
    Connected {
        profile: ProfileId,
        username: String,
    },
    Disconnected {
        profile: ProfileId,
    },
    Failed {
        profile: ProfileId,
        error: String,
    },
}

struct Trakt {
    db: Db,
    client: Client,
    ev_tx: Sender<Event>,
    repaint: Repaint,
    last_post: Option<Instant>,
    pending: Option<auth::Pending>,
}

pub(super) fn spawn(db_path: &Path, ev_tx: Sender<Event>, repaint: Repaint) -> Result<Sender<Job>> {
    let mut t = Trakt {
        db: Db::open(db_path).context("open Trakt database")?,
        client: crate::meta::client()?,
        ev_tx,
        repaint,
        last_post: None,
        pending: None,
    };
    let (tx, rx) = channel();
    thread::Builder::new()
        .name("trakt".into())
        .spawn(move || {
            t.flush_all();
            loop {
                match rx.recv_timeout(TICK) {
                    Ok(job) => t.handle(job),
                    Err(RecvTimeoutError::Timeout) => {}
                    Err(RecvTimeoutError::Disconnected) => break,
                }
                t.poll_pending();
            }
        })
        .context("spawn Trakt thread")?;
    Ok(tx)
}

impl Trakt {
    fn emit(&self, update: TraktUpdate) {
        let _ = self.ev_tx.send(Event::Trakt(update));
        (self.repaint)();
    }

    fn handle(&mut self, job: Job) {
        match job {
            Job::Connect(profile) => self.connect(profile),
            Job::CancelConnect => self.cancel_connect(),
            Job::Disconnect(profile) => self.disconnect(profile),
            Job::Scrobble {
                track,
                series_key,
                episode,
                action,
                progress,
            } => self.scrobble(&track, series_key, episode, action, progress),
            Job::History {
                profile,
                source,
                series_key,
                season,
                number,
                watched,
            } => self.history(profile, source, &series_key, season, number, watched),
        }
    }
}

fn now_secs() -> i64 {
    Utc::now().timestamp()
}

fn now_rfc3339() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}
