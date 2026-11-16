use super::{NAME, countries};
use crate::model::{Broadcast, Match, MatchStatus};
use crate::sports::lang;
use anyhow::{Context, Result};
use chrono::{NaiveDateTime, TimeZone, Utc};
use serde::Deserialize;

#[derive(Deserialize)]
struct Day {
    events: Option<Vec<Event>>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct Event {
    #[serde(rename = "idEvent")]
    id: Option<String>,
    #[serde(rename = "idLeague")]
    league: Option<String>,
    #[serde(rename = "strEvent")]
    name: Option<String>,
    #[serde(rename = "strHomeTeam")]
    home: Option<String>,
    #[serde(rename = "strAwayTeam")]
    away: Option<String>,
    #[serde(rename = "strTimestamp")]
    timestamp: Option<String>,
    #[serde(rename = "dateEvent")]
    date: Option<String>,
    #[serde(rename = "strTime")]
    time: Option<String>,
    #[serde(rename = "strStatus")]
    status: Option<String>,
}

#[derive(Deserialize)]
struct Tv {
    tvevent: Option<Vec<TvEvent>>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct TvEvent {
    #[serde(rename = "strChannel")]
    channel: Option<String>,
    #[serde(rename = "strCountry")]
    country: Option<String>,
}

fn clean(s: &Option<String>) -> &str {
    s.as_deref().unwrap_or("").trim()
}

fn kickoff(e: &Event) -> Option<i64> {
    let parse = |s: &str| NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S").ok();
    let stamp = clean(&e.timestamp);
    let stamp = stamp.strip_suffix("+00:00").unwrap_or(stamp);
    let naive =
        parse(stamp).or_else(|| parse(&format!("{}T{}", clean(&e.date), clean(&e.time))))?;
    Some(Utc.from_utc_datetime(&naive).timestamp())
}

fn to_match(e: Event, leagues: &[(i64, &str)]) -> Option<Match> {
    let league: i64 = clean(&e.league).parse().ok()?;
    let competition = leagues.iter().find(|(id, _)| *id == league)?.1;
    let id = clean(&e.id);
    if id.is_empty() {
        return None;
    }
    let (home, away) = match (clean(&e.home), clean(&e.away)) {
        (h, a) if !h.is_empty() && !a.is_empty() => (h.to_string(), a.to_string()),
        _ => (clean(&e.name).to_string(), String::new()),
    };
    if home.is_empty() {
        return None;
    }
    // shortcut: the free tier has no live state, so events show as upcoming until they drop off 3 h after start; use strStatus/livescore endpoints if a paid key is ever added
    let status = match clean(&e.status) {
        "Match Finished" | "FT" | "Finished" | "AET" | "AP" => MatchStatus::Finished,
        _ => MatchStatus::Upcoming,
    };
    Some(Match {
        key: format!("sportsdb:{id}"),
        competition: competition.to_string(),
        home,
        away,
        kickoff_utc: kickoff(&e)?,
        status,
        broadcasts: Vec::new(),
    })
}

pub fn parse_day(body: &str, leagues: &[(i64, &str)]) -> Result<Vec<Match>> {
    let day: Day = serde_json::from_str(body).context("TheSportsDB day json")?;
    Ok(day
        .events
        .unwrap_or_default()
        .into_iter()
        .filter_map(|e| to_match(e, leagues))
        .collect())
}

pub fn parse_tv(body: &str) -> Result<Vec<Broadcast>> {
    let root: Tv = serde_json::from_str(body).context("TheSportsDB tv json")?;
    let mut out: Vec<Broadcast> = Vec::new();
    for t in root.tvevent.unwrap_or_default() {
        let channel = clean(&t.channel);
        let Some(iso) = countries::iso(clean(&t.country)) else {
            continue;
        };
        if channel.is_empty() || out.iter().any(|b| b.channel == channel && b.country == iso) {
            continue;
        }
        out.push(Broadcast {
            channel: channel.to_string(),
            country: iso.to_string(),
            lang: lang::for_channel(channel, iso),
            provider: NAME.into(),
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const DAY_FIXTURE: &str = include_str!("../../../tests/fixtures/sports/sportsdb_day.json");
    const TV_FIXTURE: &str = include_str!("../../../tests/fixtures/sports/sportsdb_tv.json");
    const F1: &[(i64, &str)] = &[(4370, "Formula 1")];

    #[test]
    fn day_fixture() {
        let ms = parse_day(DAY_FIXTURE, F1).unwrap();
        assert_eq!(ms.len(), 1);
        let m = &ms[0];
        assert_eq!(m.key, "sportsdb:2408216");
        assert_eq!(m.competition, "Formula 1");
        assert_eq!(m.home, "Las Vegas Grand Prix");
        assert_eq!(m.away, "");
        assert_eq!(m.kickoff_utc, 1_795_320_000);
        assert_eq!(m.status, MatchStatus::Upcoming);
    }

    #[test]
    fn tv_fixture() {
        let b = parse_tv(TV_FIXTURE).unwrap();
        assert!(!b.is_empty());
        assert!(b.iter().all(|b| b.country == "ES" && b.provider == NAME));
        assert_eq!(b[0].lang.as_deref(), Some("es"));
    }

    #[test]
    fn null_events_is_empty() {
        assert!(parse_day(r#"{"events":null}"#, F1).unwrap().is_empty());
    }

    #[test]
    fn two_sided_and_fallback_time() {
        let body = r#"{"events":[{"idEvent":"7","idLeague":"4443","strEvent":"UFC 1: A vs B",
            "strHomeTeam":"A","strAwayTeam":"B","dateEvent":"2026-11-22","strTime":"04:00:00",
            "strTimestamp":null,"strStatus":"FT"},
            {"idEvent":"8","idLeague":"4443","strEvent":"x","strTimestamp":"nope"}]}"#;
        let ms = parse_day(body, &[(4443, "UFC")]).unwrap();
        assert_eq!(ms.len(), 1);
        assert_eq!((ms[0].home.as_str(), ms[0].away.as_str()), ("A", "B"));
        assert_eq!(ms[0].kickoff_utc, 1_795_320_000);
        assert_eq!(ms[0].status, MatchStatus::Finished);
    }

    #[test]
    fn plus_zero_suffix_accepted() {
        let body = r#"{"events":[{"idEvent":"7","idLeague":"4370","strEvent":"R",
            "strTimestamp":"2026-11-22T04:00:00+00:00"}]}"#;
        assert_eq!(parse_day(body, F1).unwrap()[0].kickoff_utc, 1_795_320_000);
    }

    #[test]
    fn tv_skips_unknown_country_and_dedupes() {
        let body = r#"{"tvevent":[
            {"strChannel":"A","strCountry":"Atlantis"},
            {"strChannel":"Sky","strCountry":"United Kingdom"},
            {"strChannel":"Sky","strCountry":"England"},
            {"strChannel":" ","strCountry":"Spain"}]}"#;
        let b = parse_tv(body).unwrap();
        assert_eq!(b.len(), 1);
        assert_eq!(b[0].country, "GB");
        assert!(parse_tv(r#"{"tvevent":null}"#).unwrap().is_empty());
    }
}
