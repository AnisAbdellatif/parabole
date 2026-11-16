//! Football tab job: fills the SQLite cache from the sports providers and assembles the list from it.
mod assemble;

use super::{Event, Worker};
use crate::db::Db;
use crate::model::{Match, MatchStatus, ProviderStatus};
use crate::sports::{self, lang, livescore, s365};
use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDate};
use reqwest::blocking::Client;
use std::sync::atomic::Ordering;
use std::time::Duration;

const TODAY_TTL_S: i64 = 60;
const DAY_TTL_S: i64 = 30 * 60;
const TV_TTL_S: i64 = 6 * 3600;
const TV_EMPTY_TTL_S: i64 = 3600;
const TV_HORIZON_S: i64 = 48 * 3600;
const PAST_S: i64 = 3 * 3600;
const AHEAD_DAYS: i64 = 7;
const PRUNE_S: i64 = 3 * 24 * 3600;
const PAUSE: Duration = Duration::from_millis(300);
const TV_PAUSE: Duration = Duration::from_millis(500);
const TV_MAX_ERRORS: u32 = 3;
const PROGRESS_EVERY: usize = 20;

/// What the Football tab shows. Sent several times per load: cached first, then as fetching progresses.
#[derive(Clone, Debug)]
pub struct FootballUpdate {
    pub matches: Vec<Match>,
    pub providers: Vec<ProviderStatus>,
    /// True on the last update of a load.
    pub done: bool,
}

pub(super) fn day_key(provider_name: &str, date: NaiveDate) -> String {
    format!("{provider_name}:day:{date}")
}

pub(super) fn tv_key(game_id: i64, country: &str) -> String {
    format!("{}:tv:{game_id}:{country}", s365::NAME)
}

fn date_of(ts: i64) -> NaiveDate {
    DateTime::from_timestamp(ts, 0)
        .unwrap_or_default()
        .date_naive()
}

/// Every UTC date from `PAST_S` ago to `AHEAD_DAYS` ahead, inclusive, ascending.
pub(super) fn days(now: i64) -> Vec<NaiveDate> {
    let last = date_of(now + AHEAD_DAYS * 86400);
    date_of(now - PAST_S)
        .iter_days()
        .take_while(|d| *d <= last)
        .collect()
}

type FetchDay = fn(&Client, NaiveDate) -> Result<Vec<Match>>;

impl Worker {
    pub(in crate::worker) fn football(&mut self, langs: Vec<String>) -> Result<()> {
        if self.football_busy.swap(true, Ordering::SeqCst) {
            return Ok(());
        }
        let path = self.db_path.clone();
        let tx = self.ev_tx.clone();
        let repaint = self.repaint.clone();
        let busy = self.football_busy.clone();
        let spawned = std::thread::Builder::new()
            .name("football".into())
            .spawn(move || {
                let emit = |u: FootballUpdate| {
                    let _ = tx.send(Event::Football(u));
                    repaint();
                };
                match Db::open(&path) {
                    Ok(db) => load(&db, &langs, &emit),
                    Err(e) => emit(FootballUpdate {
                        matches: vec![],
                        providers: vec![ProviderStatus {
                            name: "cache".into(),
                            ok: false,
                            error: Some(format!("{e:#}")),
                        }],
                        done: true,
                    }),
                }
                busy.store(false, Ordering::SeqCst);
            });
        if let Err(e) = spawned {
            self.football_busy.store(false, Ordering::SeqCst);
            return Err(e).context("spawn football thread");
        }
        Ok(())
    }
}

fn status(name: &str, error: &Option<String>) -> ProviderStatus {
    ProviderStatus {
        name: name.into(),
        ok: error.is_none(),
        error: error.clone(),
    }
}

fn statuses(s365_err: &Option<String>, ls_err: &Option<String>) -> Vec<ProviderStatus> {
    vec![
        status(s365::NAME, s365_err),
        status(livescore::NAME, ls_err),
    ]
}

/// Fetches every stale day list of one provider into the cache; returns its first error text.
fn fetch_days(
    db: &Db,
    client: &Client,
    (name, schema, fetch): (&str, i64, FetchDay),
    now: i64,
    first_request: &mut bool,
) -> Option<String> {
    let today = date_of(now);
    let mut error = None;
    for date in days(now) {
        let ttl = if date == today {
            TODAY_TTL_S
        } else {
            DAY_TTL_S
        };
        let key = day_key(name, date);
        if matches!(db.sports_cache_get(&key, schema, ttl, now), Ok(Some(_))) {
            continue;
        }
        if !std::mem::take(first_request) {
            std::thread::sleep(PAUSE);
        }
        let res = fetch(client, date).and_then(|m| {
            let json = serde_json::to_string(&m)?;
            db.sports_cache_put(&key, schema, now, &json)
        });
        if let Err(e) = res
            && error.is_none()
        {
            error = Some(format!("{e:#}"));
        }
    }
    error
}

fn load(db: &Db, langs: &[String], emit: &dyn Fn(FootballUpdate)) {
    let now = chrono::Utc::now().timestamp();
    let _ = db.sports_cache_prune(PRUNE_S, now);
    let update = |providers: Vec<ProviderStatus>, done: bool| FootballUpdate {
        matches: assemble::assemble(db, langs, now),
        providers,
        done,
    };
    emit(update(vec![], false));
    let client = match sports::client() {
        Ok(c) => c,
        Err(e) => {
            let err = Some(format!("{e:#}"));
            emit(update(statuses(&err, &err), true));
            return;
        }
    };
    let mut first = true;
    let mut s365_err = fetch_days(
        db,
        &client,
        (s365::NAME, s365::SCHEMA, s365::fetch_day),
        now,
        &mut first,
    );
    let ls_err = fetch_days(
        db,
        &client,
        (livescore::NAME, livescore::SCHEMA, livescore::fetch_day),
        now,
        &mut first,
    );
    emit(update(statuses(&s365_err, &ls_err), false));

    let candidates: Vec<i64> = assemble::assemble(db, langs, now)
        .iter()
        .filter(|m| m.status != MatchStatus::Finished && m.kickoff_utc <= now + TV_HORIZON_S)
        .filter_map(|m| s365::game_id(&m.key))
        .collect();
    let countries = lang::countries_for(langs);
    let (mut requests, mut consecutive) = (0usize, 0u32);
    'tv: for id in candidates {
        for country in &countries {
            let key = tv_key(id, country);
            // 365scores fills most countries' listings only on match day, so re-ask empty ones sooner.
            let fresh = match db.sports_cache_get(&key, s365::SCHEMA, TV_TTL_S, now) {
                Ok(Some(json)) => {
                    json != "[]"
                        || matches!(
                            db.sports_cache_get(&key, s365::SCHEMA, TV_EMPTY_TTL_S, now),
                            Ok(Some(_))
                        )
                }
                _ => false,
            };
            if fresh {
                continue;
            }
            std::thread::sleep(TV_PAUSE);
            let res = s365::fetch_tv(&client, id, country).and_then(|list| {
                let json = serde_json::to_string(&list)?;
                db.sports_cache_put(&key, s365::SCHEMA, now, &json)
            });
            match res {
                Ok(()) => consecutive = 0,
                Err(e) => {
                    if s365_err.is_none() {
                        s365_err = Some(format!("{e:#}"));
                    }
                    consecutive += 1;
                    if consecutive >= TV_MAX_ERRORS {
                        break 'tv;
                    }
                }
            }
            requests += 1;
            if requests % PROGRESS_EVERY == 0 {
                emit(update(statuses(&s365_err, &ls_err), false));
            }
        }
    }
    emit(update(statuses(&s365_err, &ls_err), true));
}

#[cfg(test)]
mod tests;
