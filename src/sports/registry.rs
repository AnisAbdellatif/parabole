//! The list of sports and where each gets its data. Adding a sport: one file in `catalog/` plus one line in `SPORTS`.
use super::catalog;
use crate::model::{Broadcast, Match};
use anyhow::Result;
use chrono::NaiveDate;
use reqwest::blocking::Client;
use std::time::Duration;

pub type FetchDay = fn(&Client, NaiveDate) -> Result<Vec<Match>>;

pub struct DaySource {
    pub provider: &'static str,
    pub schema: i64,
    pub fetch: FetchDay,
}

pub enum TvSource {
    /// One request per match and country (ISO alpha-2).
    PerCountry(fn(&Client, &Match, &str) -> Result<Vec<Broadcast>>),
    /// One request per match returning every country; the worker keeps the user's countries.
    AllCountries(fn(&Client, &Match) -> Result<Vec<Broadcast>>),
}

pub struct Sport {
    /// Settings and cache-key prefix, e.g. "basketball".
    pub id: &'static str,
    pub name: &'static str,
    /// The first source is the base list; later ones only add channels via `merge::attach`.
    pub days: &'static [DaySource],
    pub tv: Option<TvSource>,
    /// Pause before each TV request (provider rate limits).
    pub tv_pause: Duration,
}

pub const SPORTS: &[&Sport] = &[
    &catalog::football::SPORT,
    &catalog::basketball::SPORT,
    &catalog::tennis::SPORT,
];

pub fn find(id: &str) -> Option<&'static Sport> {
    SPORTS.iter().copied().find(|s| s.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_is_consistent() {
        let mut ids: Vec<_> = SPORTS.iter().map(|s| s.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), SPORTS.len());
        assert_eq!(find("football").map(|s| s.name), Some("Football"));
        assert!(find("nope").is_none());
        assert!(SPORTS.iter().all(|s| !s.days.is_empty()));
    }
}
