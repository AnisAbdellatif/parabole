//! Basketball: 365scores only.
use crate::model::Match;
use crate::sports::registry::{DaySource, Sport, TvSource};
use crate::sports::s365;
use anyhow::Result;
use chrono::NaiveDate;
use reqwest::blocking::Client;
use std::time::Duration;

fn day(c: &Client, d: NaiveDate) -> Result<Vec<Match>> {
    s365::fetch_day(
        c,
        2,
        &s365::Pick::Competitions(&[(103, "NBA"), (569, "EuroLeague")]),
        d,
    )
}

pub const SPORT: Sport = Sport {
    id: "basketball",
    name: "Basketball",
    days: &[DaySource {
        provider: s365::NAME,
        schema: s365::SCHEMA,
        fetch: day,
    }],
    tv: Some(TvSource::PerCountry(s365::fetch_tv)),
    tv_pause: Duration::from_millis(500),
};
