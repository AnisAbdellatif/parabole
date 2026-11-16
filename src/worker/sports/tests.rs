use super::assemble::assemble;
use super::*;
use crate::model::Broadcast;
use crate::sports::registry::{DaySource, Sport, TvSource};
use crate::sports::{catalog, livescore, s365};
use reqwest::blocking::Client;
use std::time::Duration;

const NOW: i64 = 1_800_000_000; // 2027-01-15 08:00 UTC
const H: i64 = 3600;
const FOOTBALL: Sport = catalog::football::SPORT;

fn m(key: &str, home: &str, away: &str, kickoff: i64, status: MatchStatus) -> Match {
    Match {
        key: key.into(),
        competition: "League".into(),
        home: home.into(),
        away: away.into(),
        kickoff_utc: kickoff,
        status,
        broadcasts: vec![],
    }
}

fn live() -> MatchStatus {
    MatchStatus::Live {
        clock: "10'".into(),
        home: "0".into(),
        away: "0".into(),
    }
}

fn bc(channel: &str, country: &str, provider: &str) -> Broadcast {
    Broadcast {
        channel: channel.into(),
        country: country.into(),
        lang: None,
        provider: provider.into(),
    }
}

fn put_day(db: &Db, name: &str, schema: i64, ms: &[Match]) {
    let key = day_key("football", name, date_of(NOW));
    db.sports_cache_put(&key, schema, NOW, &serde_json::to_string(ms).unwrap())
        .unwrap();
}

fn put_s365(db: &Db, ms: &[Match]) {
    put_day(db, s365::NAME, s365::SCHEMA, ms);
}

fn keys(v: &[Match]) -> Vec<&str> {
    v.iter().map(|m| m.key.as_str()).collect()
}

fn langs(codes: &[&str]) -> Vec<String> {
    codes.iter().map(|c| c.to_string()).collect()
}

#[test]
fn drops_finished_and_stale_upcoming_keeps_live() {
    let db = Db::open_in_memory().unwrap();
    put_s365(
        &db,
        &[
            m("s365:1", "A", "B", NOW + H, MatchStatus::Finished),
            m("s365:2", "C", "D", NOW - 4 * H, MatchStatus::Upcoming),
            m("s365:3", "E", "F", NOW - 2 * H, MatchStatus::Upcoming),
            m("s365:4", "G", "H", NOW - 10 * H, live()),
        ],
    );
    let out = assemble(&db, &FOOTBALL, &langs(&["en"]), NOW);
    assert_eq!(keys(&out), ["s365:4", "s365:3"]);
}

#[test]
fn tv_cache_follows_selected_countries() {
    let db = Db::open_in_memory().unwrap();
    put_s365(
        &db,
        &[m("s365:7", "A", "B", NOW + H, MatchStatus::Upcoming)],
    );
    let country = lang::countries_for(&langs(&["en"]))[0];
    let other = lang::countries_for(&langs(&["fr"]))[0];
    for (c, ch) in [(country, "Sky"), (other, "Canal")] {
        let json = serde_json::to_string(&[bc(ch, c, s365::NAME)]).unwrap();
        db.sports_cache_put(
            &tv_key("football", "s365:7", Some(c)),
            s365::SCHEMA,
            NOW,
            &json,
        )
        .unwrap();
    }
    let out = assemble(&db, &FOOTBALL, &langs(&["en"]), NOW);
    let chans: Vec<_> = out[0]
        .broadcasts
        .iter()
        .map(|b| b.channel.as_str())
        .collect();
    assert_eq!(chans, ["Sky"]);
}

#[test]
fn livescore_channels_attached() {
    let db = Db::open_in_memory().unwrap();
    put_s365(
        &db,
        &[m(
            "s365:1",
            "Arsenal",
            "Chelsea",
            NOW + H,
            MatchStatus::Upcoming,
        )],
    );
    let mut ls = m(
        "livescore:9",
        "Arsenal",
        "Chelsea",
        NOW + H + 600,
        MatchStatus::Upcoming,
    );
    ls.broadcasts.push(bc("BT", "GB", livescore::NAME));
    put_day(&db, livescore::NAME, livescore::SCHEMA, &[ls]);
    let out = assemble(&db, &FOOTBALL, &langs(&["en"]), NOW);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].broadcasts.len(), 1);
}

#[test]
fn sorts_live_first_then_kickoff() {
    let db = Db::open_in_memory().unwrap();
    put_s365(
        &db,
        &[
            m("s365:1", "A", "B", NOW + H, MatchStatus::Upcoming),
            m("s365:2", "C", "D", NOW + 5 * H, live()),
            m("s365:3", "E", "F", NOW + 2 * H, MatchStatus::Upcoming),
        ],
    );
    assert_eq!(
        keys(&assemble(&db, &FOOTBALL, &langs(&["en"]), NOW)),
        ["s365:2", "s365:1", "s365:3"]
    );
}

#[test]
fn wrong_schema_ignored() {
    let db = Db::open_in_memory().unwrap();
    put_day(
        &db,
        s365::NAME,
        s365::SCHEMA + 1,
        &[m("s365:1", "A", "B", NOW + H, MatchStatus::Upcoming)],
    );
    assert!(assemble(&db, &FOOTBALL, &langs(&["en"]), NOW).is_empty());
}

#[test]
fn days_bounds_and_keys() {
    let d = days(NOW);
    assert_eq!(d.first(), Some(&date_of(NOW - PAST_S)));
    assert_eq!(d.last(), Some(&date_of(NOW + 7 * 86400)));
    assert_eq!(d.len(), 8);
    let date = NaiveDate::from_ymd_opt(2027, 1, 15).unwrap();
    assert_eq!(day_key("x", "P", date), "x:P:day:2027-01-15");
    assert_eq!(tv_key("x", "s365:5", Some("GB")), "x:tv:s365:5:GB");
    assert_eq!(tv_key("x", "s365:5", None), "x:tv:s365:5");
}

fn no_fetch(_: &Client, _: NaiveDate) -> Result<Vec<Match>> {
    Ok(vec![])
}

fn no_tv(_: &Client, _: &Match) -> Result<Vec<Broadcast>> {
    Ok(vec![])
}

#[test]
fn all_countries_tv_filtered_from_cache() {
    const FAKE: Sport = Sport {
        id: "fake",
        name: "Fake",
        days: &[DaySource {
            provider: "P",
            schema: 1,
            fetch: no_fetch,
        }],
        tv: Some(TvSource::AllCountries(no_tv)),
        tv_pause: Duration::ZERO,
    };
    let db = Db::open_in_memory().unwrap();
    let ms = [m("p:1", "A", "B", NOW + H, MatchStatus::Upcoming)];
    db.sports_cache_put(
        &day_key("fake", "P", date_of(NOW)),
        1,
        NOW,
        &serde_json::to_string(&ms).unwrap(),
    )
    .unwrap();
    let country = lang::countries_for(&langs(&["en"]))[0];
    let other = lang::countries_for(&langs(&["fr"]))[0];
    let all = [bc("Sky", country, "P"), bc("Canal", other, "P")];
    db.sports_cache_put(
        &tv_key("fake", "p:1", None),
        1,
        NOW,
        &serde_json::to_string(&all).unwrap(),
    )
    .unwrap();
    let out = assemble(&db, &FAKE, &langs(&["en"]), NOW);
    let chans: Vec<_> = out[0]
        .broadcasts
        .iter()
        .map(|b| b.channel.as_str())
        .collect();
    assert_eq!(chans, ["Sky"]);
}
