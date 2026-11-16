//! Tennis: 365scores only.
use crate::model::Match;
use crate::sports::registry::{DaySource, Sport, TvSource};
use crate::sports::s365;
use anyhow::Result;
use chrono::NaiveDate;
use reqwest::blocking::Client;
use std::time::Duration;

// 88 ATP, 87 WTA, 53 Grand Slams (365scores tennis "countries"); tournaments change weekly so ids can't be allowlisted.
fn day(c: &Client, d: NaiveDate) -> Result<Vec<Match>> {
    s365::fetch_day(c, 3, &s365::Pick::Categories(&[88, 87, 53]), d)
}

pub const SPORT: Sport = Sport {
    id: "tennis",
    name: "Tennis",
    days: &[DaySource {
        provider: s365::NAME,
        schema: s365::SCHEMA,
        fetch: day,
    }],
    tv: Some(TvSource::PerCountry(s365::fetch_tv)),
    tv_pause: Duration::from_millis(500),
};
