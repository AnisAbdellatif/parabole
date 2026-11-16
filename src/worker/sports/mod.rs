//! Sports tab job: fills the SQLite cache from the sport's providers and assembles the list from it.
mod assemble;
mod fetch;

use super::{Event, Worker};
use crate::db::Db;
use crate::model::{Match, MatchStatus, ProviderStatus};
use crate::sports::registry::{self, Sport};
use crate::sports::{self, lang};
use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDate};
use std::sync::atomic::Ordering;

const TV_HORIZON_S: i64 = 48 * 3600;
const PAST_S: i64 = 3 * 3600;
const AHEAD_DAYS: i64 = 7;
const PRUNE_S: i64 = 3 * 24 * 3600;
const PROGRESS_EVERY: usize = 20;

/// What the Sports tab shows. Sent several times per load: cached first, then as fetching progresses.
#[derive(Clone, Debug)]
pub struct SportsUpdate {
    /// Registry id of the sport this update belongs to.
    pub sport: String,
    pub matches: Vec<Match>,
    pub providers: Vec<ProviderStatus>,
    /// True on the last update of a load.
    pub done: bool,
}

pub(super) fn day_key(sport_id: &str, provider_name: &str, date: NaiveDate) -> String {
    format!("{sport_id}:{provider_name}:day:{date}")
}

/// `country` is `Some` for per-country sources, `None` when one entry holds every country.
pub(super) fn tv_key(sport_id: &str, match_key: &str, country: Option<&str>) -> String {
    match country {
        Some(c) => format!("{sport_id}:tv:{match_key}:{c}"),
        None => format!("{sport_id}:tv:{match_key}"),
    }
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

fn failed(sport: &str, name: &str, error: String) -> SportsUpdate {
    SportsUpdate {
        sport: sport.into(),
        matches: vec![],
        providers: vec![ProviderStatus {
            name: name.into(),
            ok: false,
            error: Some(error),
        }],
        done: true,
    }
}

impl Worker {
    pub(in crate::worker) fn sports(&mut self, sport: String, langs: Vec<String>) -> Result<()> {
        let tx = self.ev_tx.clone();
        let repaint = self.repaint.clone();
        let Some(def) = registry::find(&sport) else {
            let _ = tx.send(Event::Sports(failed(
                &sport,
                "sports",
                format!("unknown sport {sport}"),
            )));
            repaint();
            return Ok(());
        };
        if self.sports_busy.swap(true, Ordering::SeqCst) {
            return Ok(());
        }
        let path = self.db_path.clone();
        let busy = self.sports_busy.clone();
        let spawned = std::thread::Builder::new()
            .name("sports".into())
            .spawn(move || {
                let emit = |u: SportsUpdate| {
                    let _ = tx.send(Event::Sports(u));
                    repaint();
                };
                match Db::open(&path) {
                    Ok(db) => load(&db, def, &langs, &emit),
                    Err(e) => emit(failed(def.id, "cache", format!("{e:#}"))),
                }
                busy.store(false, Ordering::SeqCst);
            });
        if let Err(e) = spawned {
            self.sports_busy.store(false, Ordering::SeqCst);
            return Err(e).context("spawn sports thread");
        }
        Ok(())
    }
}

fn statuses(sport: &Sport, errors: &[Option<String>]) -> Vec<ProviderStatus> {
    sport
        .days
        .iter()
        .zip(errors)
        .map(|(src, error)| ProviderStatus {
            name: src.provider.into(),
            ok: error.is_none(),
            error: error.clone(),
        })
        .collect()
}

fn load(db: &Db, sport: &'static Sport, langs: &[String], emit: &dyn Fn(SportsUpdate)) {
    let now = chrono::Utc::now().timestamp();
    let _ = db.sports_cache_prune(PRUNE_S, now);
    let update = |providers: Vec<ProviderStatus>, done: bool| SportsUpdate {
        sport: sport.id.into(),
        matches: assemble::assemble(db, sport, langs, now),
        providers,
        done,
    };
    emit(update(vec![], false));
    let client = match sports::client() {
        Ok(c) => c,
        Err(e) => {
            let errors = vec![Some(format!("{e:#}")); sport.days.len()];
            emit(update(statuses(sport, &errors), true));
            return;
        }
    };
    let mut errors = fetch::fetch_days(db, &client, sport, now);
    emit(update(statuses(sport, &errors), false));

    let candidates: Vec<Match> = assemble::assemble(db, sport, langs, now)
        .into_iter()
        .filter(|m| m.status != MatchStatus::Finished && m.kickoff_utc <= now + TV_HORIZON_S)
        .collect();
    let countries = lang::countries_for(langs);
    fetch::fetch_tv(
        db,
        &client,
        sport,
        &candidates,
        &countries,
        now,
        &mut errors,
        &|errors| emit(update(statuses(sport, errors), false)),
    );
    emit(update(statuses(sport, &errors), true));
}

#[cfg(test)]
mod tests;
