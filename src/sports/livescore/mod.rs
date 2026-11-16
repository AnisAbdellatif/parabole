//! LiveScore (prod-public-api.livescore.com): every match of a day with status and TV channels
//! (mostly GB, IE, US, CA). Used only to add channels to 365scores matches.
mod parse;
pub use parse::parse_day;

use super::get_text;
use crate::model::Match;
use anyhow::{Context, Result};
use chrono::NaiveDate;
use reqwest::blocking::Client;

pub const NAME: &str = "LiveScore";
pub const SCHEMA: i64 = 1;

pub fn day_url(date: NaiveDate) -> String {
    format!(
        "https://prod-public-api.livescore.com/v1/api/app/date/soccer/{}/0?MD=1",
        date.format("%Y%m%d")
    )
}

pub fn fetch_day(client: &Client, date: NaiveDate) -> Result<Vec<Match>> {
    (|| parse_day(&get_text(client, &day_url(date))?))().context("LiveScore day")
}
