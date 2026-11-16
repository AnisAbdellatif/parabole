//! Football: 365scores is the base list, LiveScore adds channels.
use crate::model::Match;
use crate::sports::registry::{DaySource, Sport, TvSource};
use crate::sports::{livescore, s365};
use anyhow::Result;
use chrono::NaiveDate;
use reqwest::blocking::Client;
use std::time::Duration;

/// Competition id → display name. This is the "popular competitions" allowlist.
pub const COMPETITIONS: &[(i64, &str)] = &[
    (7, "Premier League"),
    (11, "LaLiga"),
    (17, "Serie A"),
    (25, "Bundesliga"),
    (35, "Ligue 1"),
    (572, "Champions League"),
    (573, "Europa League"),
    (7685, "Conference League"),
    (472, "UEFA Super Cup"),
    (7016, "Nations League"),
    (5930, "World Cup"),
    (5421, "World Cup qualifiers (UEFA)"),
    (613, "World Cup qualifiers (CONMEBOL)"),
    (6316, "Euro"),
    (6071, "Euro qualifiers"),
    (595, "Copa América"),
    (102, "Copa Libertadores"),
    (8, "FA Cup"),
    (13, "Copa del Rey"),
    (20, "Coppa Italia"),
    (28, "DFB-Pokal"),
    (37, "Coupe de France"),
];

fn day(c: &Client, d: NaiveDate) -> Result<Vec<Match>> {
    s365::fetch_day(c, 1, &s365::Pick::Competitions(COMPETITIONS), d)
}

pub const SPORT: Sport = Sport {
    id: "football",
    name: "Football",
    days: &[
        DaySource {
            provider: s365::NAME,
            schema: s365::SCHEMA,
            fetch: day,
        },
        DaySource {
            provider: livescore::NAME,
            schema: livescore::SCHEMA,
            fetch: livescore::fetch_day,
        },
    ],
    tv: Some(TvSource::PerCountry(s365::fetch_tv)),
    tv_pause: Duration::from_millis(500),
};
