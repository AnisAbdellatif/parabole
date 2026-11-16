use super::NAME;
use crate::model::{Broadcast, Match, MatchStatus};
use crate::sports::lang;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
struct Root {
    #[serde(rename = "Stages")]
    stages: Vec<Stage>,
}

#[derive(Deserialize)]
struct Stage {
    #[serde(default, rename = "CompN")]
    comp_n: Option<String>,
    #[serde(default, rename = "Cnm")]
    cnm: Option<String>,
    #[serde(default, rename = "Events")]
    events: Vec<serde_json::Value>,
}

#[derive(Deserialize)]
struct Event {
    #[serde(rename = "Eid")]
    id: String,
    /// YYYYMMDDhhmmss, UTC.
    #[serde(rename = "Esd")]
    start: i64,
    #[serde(default, rename = "Eps")]
    eps: String,
    /// 0 not started, 1 live, 2 finished, 3 cancelled.
    #[serde(default, rename = "Epr")]
    epr: i64,
    #[serde(default, rename = "Tr1")]
    tr1: Option<String>,
    #[serde(default, rename = "Tr2")]
    tr2: Option<String>,
    #[serde(rename = "T1")]
    t1: Vec<Team>,
    #[serde(rename = "T2")]
    t2: Vec<Team>,
    #[serde(default, rename = "Media")]
    media: HashMap<String, Vec<serde_json::Value>>,
}

#[derive(Deserialize)]
struct Team {
    #[serde(rename = "Nm")]
    name: String,
}

#[derive(Deserialize)]
struct Media {
    #[serde(rename = "eventId")]
    name: String,
    #[serde(rename = "type")]
    kind: String,
    #[serde(default, rename = "allowedCountries")]
    countries: Vec<String>,
}

fn goals(s: &Option<String>) -> String {
    s.as_deref()
        .and_then(|s| s.trim().parse::<u16>().ok())
        .unwrap_or(0)
        .to_string()
}

fn broadcasts(media: HashMap<String, Vec<serde_json::Value>>) -> Vec<Broadcast> {
    let mut out: Vec<Broadcast> = Vec::new();
    for v in media.into_values().flatten() {
        let Ok(m) = serde_json::from_value::<Media>(v) else {
            continue;
        };
        let channel = m.name.trim();
        if m.kind != "TV_CHANNEL" || channel.is_empty() || crate::sports::is_placeholder(channel) {
            continue;
        }
        for c in &m.countries {
            let country = c.to_ascii_uppercase();
            let dup = out.iter().any(|b| {
                b.country == country && b.channel.to_lowercase() == channel.to_lowercase()
            });
            if !dup {
                out.push(Broadcast {
                    channel: channel.to_string(),
                    lang: lang::for_channel(channel, &country),
                    country,
                    provider: NAME.into(),
                });
            }
        }
    }
    out.sort_by(|a, b| (&a.country, &a.channel).cmp(&(&b.country, &b.channel)));
    out
}

fn to_match(e: Event, competition: &str) -> Option<Match> {
    let home = e.t1.into_iter().next()?.name;
    let away = e.t2.into_iter().next()?.name;
    let kickoff_utc = chrono::NaiveDateTime::parse_from_str(&e.start.to_string(), "%Y%m%d%H%M%S")
        .ok()?
        .and_utc()
        .timestamp();
    let status = match e.epr {
        0 => MatchStatus::Upcoming,
        1 => MatchStatus::Live {
            clock: e.eps,
            home: goals(&e.tr1),
            away: goals(&e.tr2),
        },
        _ => MatchStatus::Finished,
    };
    Some(Match {
        key: format!("livescore:{}", e.id),
        competition: competition.to_string(),
        home,
        away,
        kickoff_utc,
        status,
        broadcasts: broadcasts(e.media),
    })
}

pub fn parse_day(body: &str) -> Result<Vec<Match>> {
    let root: Root = serde_json::from_str(body).context("LiveScore day json")?;
    let mut out = Vec::new();
    for stage in root.stages {
        let comp = stage.comp_n.or(stage.cnm).unwrap_or_default();
        for v in stage.events {
            if let Some(m) = serde_json::from_value::<Event>(v)
                .ok()
                .and_then(|e| to_match(e, &comp))
            {
                out.push(m);
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!("../../../tests/fixtures/sports/livescore_day.json");

    fn event(extra: &str) -> String {
        format!(
            r#"{{"Stages":[{{"CompN":"Premier League","Events":[{{"Eid":"7","Esd":20261116194500,{extra}
            "T1":[{{"Nm":"A"}}],"T2":[{{"Nm":"B"}}]}}]}}]}}"#
        )
    }

    #[test]
    fn status_mapping() {
        let up = parse_day(&event(r#""Epr":0,"#)).unwrap();
        assert_eq!(up[0].status, MatchStatus::Upcoming);
        let live = parse_day(&event(r#""Epr":1,"Eps":"79'","Tr1":"2","Tr2":"x","#)).unwrap();
        assert_eq!(
            live[0].status,
            MatchStatus::Live {
                clock: "79'".into(),
                home: "2".into(),
                away: "0".into()
            }
        );
        for epr in [2, 3] {
            let ms = parse_day(&event(&format!(r#""Epr":{epr},"#))).unwrap();
            assert_eq!(ms[0].status, MatchStatus::Finished);
        }
        assert_eq!(up[0].key, "livescore:7");
        assert_eq!(up[0].competition, "Premier League");
    }

    #[test]
    fn start_time_is_utc() {
        let ms = parse_day(&event("")).unwrap();
        let want = chrono::NaiveDate::from_ymd_opt(2026, 11, 16)
            .and_then(|d| d.and_hms_opt(19, 45, 0))
            .map(|t| t.and_utc().timestamp());
        assert_eq!(Some(ms[0].kickoff_utc), want);
    }

    #[test]
    fn media_dedupes_and_filters() {
        let media = r#""Media":{"1":[{"eventId":" Sky ","type":"TV_CHANNEL","allowedCountries":["US","US","gb"]},
            {"eventId":"Site","type":"STREAM","allowedCountries":["US"]}]},"#;
        let ms = parse_day(&event(media)).unwrap();
        let b = &ms[0].broadcasts;
        assert_eq!(b.len(), 2);
        assert!(
            b.iter()
                .all(|x| x.channel == "Sky" && x.provider == "LiveScore")
        );
        assert!(
            b.iter()
                .any(|x| x.country == "GB" && x.lang.as_deref() == Some("en"))
        );
    }

    #[test]
    fn missing_stages_is_error() {
        assert!(parse_day("{}").is_err());
    }

    #[test]
    fn event_without_team_skipped() {
        let body = r#"{"Stages":[{"Events":[{"Eid":"1","Esd":20261116194500,"T1":[],"T2":[{"Nm":"B"}]}]}]}"#;
        assert!(parse_day(body).unwrap().is_empty());
    }

    #[test]
    fn day_fixture() {
        let ms = parse_day(FIXTURE).unwrap();
        assert!(!ms.is_empty());
        assert!(ms.iter().all(|m| m.key.starts_with("livescore:")));
        assert!(ms.iter().any(|m| !m.broadcasts.is_empty()));
    }
}
