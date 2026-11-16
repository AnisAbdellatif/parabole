//! Network side of a sports load: refreshes stale day lists and TV listings in the cache.
use super::{PROGRESS_EVERY, date_of, day_key, days, tv_key};
use crate::db::Db;
use crate::model::{Broadcast, Match};
use crate::sports::registry::{Sport, TvSource};
use anyhow::Result;
use reqwest::blocking::Client;
use std::time::Duration;

const TODAY_TTL_S: i64 = 60;
const DAY_TTL_S: i64 = 30 * 60;
const TV_TTL_S: i64 = 6 * 3600;
const TV_EMPTY_TTL_S: i64 = 3600;
const PAUSE: Duration = Duration::from_millis(300);
const TV_MAX_ERRORS: u32 = 3;

/// Fetches every stale day list of every source of `sport`; returns each source's first error text, in order.
pub(super) fn fetch_days(db: &Db, client: &Client, sport: &Sport, now: i64) -> Vec<Option<String>> {
    let today = date_of(now);
    let mut first_request = true;
    let mut errors = Vec::new();
    for src in sport.days {
        let mut error = None;
        for date in days(now) {
            let ttl = if date == today {
                TODAY_TTL_S
            } else {
                DAY_TTL_S
            };
            let key = day_key(sport.id, src.provider, date);
            if matches!(db.sports_cache_get(&key, src.schema, ttl, now), Ok(Some(_))) {
                continue;
            }
            if !std::mem::take(&mut first_request) {
                std::thread::sleep(PAUSE);
            }
            let res = (src.fetch)(client, date).and_then(|m| {
                let json = serde_json::to_string(&m)?;
                db.sports_cache_put(&key, src.schema, now, &json)
            });
            if let Err(e) = res
                && error.is_none()
            {
                error = Some(format!("{e:#}"));
            }
        }
        errors.push(error);
    }
    errors
}

fn request(
    tv: &TvSource,
    client: &Client,
    m: &Match,
    country: Option<&str>,
) -> Result<Vec<Broadcast>> {
    match tv {
        TvSource::PerCountry(f) => f(client, m, country.unwrap_or_default()),
        TvSource::AllCountries(f) => f(client, m),
    }
}

/// Fills the TV cache for `matches`; errors land on the first source's entry of `errors`.
#[allow(clippy::too_many_arguments)]
pub(super) fn fetch_tv(
    db: &Db,
    client: &Client,
    sport: &Sport,
    matches: &[Match],
    countries: &[&str],
    now: i64,
    errors: &mut [Option<String>],
    progress: &dyn Fn(&[Option<String>]),
) {
    let Some(tv) = &sport.tv else {
        return;
    };
    let schema = sport.days[0].schema;
    let targets: Vec<Option<&str>> = match tv {
        TvSource::PerCountry(_) => countries.iter().map(|c| Some(*c)).collect(),
        TvSource::AllCountries(_) => vec![None],
    };
    let (mut requests, mut consecutive) = (0usize, 0u32);
    for m in matches {
        for &country in &targets {
            let key = tv_key(sport.id, &m.key, country);
            // Providers fill most listings only on match day, so re-ask empty ones sooner.
            let fresh = match db.sports_cache_get(&key, schema, TV_TTL_S, now) {
                Ok(Some(json)) => {
                    json != "[]"
                        || matches!(
                            db.sports_cache_get(&key, schema, TV_EMPTY_TTL_S, now),
                            Ok(Some(_))
                        )
                }
                _ => false,
            };
            if fresh {
                continue;
            }
            std::thread::sleep(sport.tv_pause);
            let res = request(tv, client, m, country).and_then(|list| {
                let json = serde_json::to_string(&list)?;
                db.sports_cache_put(&key, schema, now, &json)
            });
            match res {
                Ok(()) => consecutive = 0,
                Err(e) => {
                    if errors[0].is_none() {
                        errors[0] = Some(format!("{e:#}"));
                    }
                    consecutive += 1;
                    if consecutive >= TV_MAX_ERRORS {
                        return;
                    }
                }
            }
            requests += 1;
            if requests % PROGRESS_EVERY == 0 {
                progress(errors);
            }
        }
    }
}
