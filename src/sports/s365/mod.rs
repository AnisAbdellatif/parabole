//! 365scores (webws.365scores.com): fixtures, live state and per-country TV channels.
mod ids;
mod parse;
pub use parse::{parse_day, parse_tv};

use super::get_text;
use crate::model::{Broadcast, Match};
use anyhow::{Context, Result};
use chrono::NaiveDate;
use reqwest::blocking::Client;

pub const NAME: &str = "365scores";
/// Bump when parsing changes so cached results from older code are ignored.
pub const SCHEMA: i64 = 1;
const BASE: &str = "https://webws.365scores.com/web/";

pub fn day_url(date: NaiveDate) -> String {
    let d = date.format("%d/%m/%Y");
    format!(
        "{BASE}games/allscores/?appTypeId=5&langId=1&timezoneName=UTC&userCountryId=1&sports=1&startDate={d}&endDate={d}"
    )
}

pub fn tv_url(game_id: i64, country_id: i64) -> String {
    format!(
        "{BASE}game/?appTypeId=5&langId=1&timezoneName=UTC&userCountryId={country_id}&gameId={game_id}"
    )
}

/// Allowlisted matches of one UTC day, without broadcasts.
pub fn fetch_day(client: &Client, date: NaiveDate) -> Result<Vec<Match>> {
    (|| parse_day(&get_text(client, &day_url(date))?))().context("365scores day")
}

/// Channels for one match in one country; empty when the country has no 365scores id.
pub fn fetch_tv(client: &Client, game_id: i64, country: &str) -> Result<Vec<Broadcast>> {
    let Some(cid) = ids::country_id(country) else {
        return Ok(vec![]);
    };
    (|| parse_tv(&get_text(client, &tv_url(game_id, cid))?, country))().context("365scores tv")
}

/// The 365scores game id inside a "s365:<id>" match key.
pub fn game_id(key: &str) -> Option<i64> {
    key.strip_prefix("s365:")?.parse().ok()
}
