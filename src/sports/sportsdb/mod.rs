//! TheSportsDB (free key "123"): fixtures for single-event sports and per-event TV channels.
mod countries;
mod parse;
pub use parse::{parse_day, parse_tv};

use super::get_text;
use crate::model::{Broadcast, Match};
use anyhow::{Context, Result};
use chrono::NaiveDate;
use reqwest::blocking::Client;
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub const NAME: &str = "TheSportsDB";
/// Bump when parsing changes so cached results from older code are ignored.
pub const SCHEMA: i64 = 2;
const BASE: &str = "https://www.thesportsdb.com/api/v1/json/123/";

// The free key allows a burst of ~10 requests, then blocks for 2 minutes; ~9/min sustained is safe.
const MIN_GAP: Duration = Duration::from_millis(6500);
static LAST: Mutex<Option<Instant>> = Mutex::new(None);

/// Every TheSportsDB request goes through here so F1, WWE and UFC share one rate limit.
fn get(client: &Client, url: &str) -> Result<String> {
    let mut last = LAST.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(t) = *last {
        std::thread::sleep(MIN_GAP.saturating_sub(t.elapsed()));
    }
    let res = get_text(client, url);
    *last = Some(Instant::now());
    res
}

pub fn day_url(sport: &str, date: NaiveDate) -> String {
    format!(
        "{BASE}eventsday.php?d={}&s={sport}",
        date.format("%Y-%m-%d")
    )
}

pub fn tv_url(event_id: i64) -> String {
    format!("{BASE}lookuptv.php?id={event_id}")
}

/// Events of the given leagues (id, display name) on one UTC day, without broadcasts.
pub fn fetch_day(
    client: &Client,
    sport: &str,
    leagues: &[(i64, &str)],
    date: NaiveDate,
) -> Result<Vec<Match>> {
    (|| parse_day(&get(client, &day_url(sport, date))?, leagues))().context("TheSportsDB day")
}

/// Channels for one event in every country; empty when the key isn't a TheSportsDB one.
pub fn fetch_tv(client: &Client, m: &Match) -> Result<Vec<Broadcast>> {
    let Some(id) = event_id(&m.key) else {
        return Ok(vec![]);
    };
    (|| parse_tv(&get(client, &tv_url(id))?))().context("TheSportsDB tv")
}

fn event_id(key: &str) -> Option<i64> {
    key.strip_prefix("sportsdb:")?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_id_from_key() {
        assert_eq!(event_id("sportsdb:2408216"), Some(2408216));
        assert_eq!(event_id("s365:5"), None);
        assert_eq!(event_id("sportsdb:x"), None);
    }
}
