use crate::model::Match;
use crate::sports::registry::{DaySource, Sport, TvSource};
use crate::sports::sportsdb;
use anyhow::Result;
use chrono::NaiveDate;
use reqwest::blocking::Client;
use std::time::Duration;

fn day(c: &Client, d: NaiveDate) -> Result<Vec<Match>> {
    sportsdb::fetch_day(c, "Fighting", &[(4443, "UFC")], d)
}

pub const SPORT: Sport = Sport {
    id: "ufc",
    name: "UFC",
    days: &[DaySource {
        provider: sportsdb::NAME,
        schema: sportsdb::SCHEMA,
        fetch: day,
    }],
    tv: Some(TvSource::AllCountries(sportsdb::fetch_tv)),
    // The free tier allows ~30 requests/min.
    tv_pause: Duration::from_millis(2500),
};
