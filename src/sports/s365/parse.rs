use super::{NAME, ids};
use crate::model::{Broadcast, Match, MatchStatus};
use crate::sports::lang;
use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Deserialize)]
struct Day {
    games: Vec<serde_json::Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Game {
    id: i64,
    competition_id: i64,
    start_time: String,
    status_group: i64,
    #[serde(default)]
    short_status_text: String,
    #[serde(default)]
    game_time_display: String,
    home_competitor: Competitor,
    away_competitor: Competitor,
}

#[derive(Deserialize)]
struct Competitor {
    name: String,
    #[serde(default)]
    score: Option<f64>,
}

#[derive(Deserialize)]
struct TvRoot {
    game: TvGame,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TvGame {
    #[serde(default)]
    tv_networks: Vec<serde_json::Value>,
}

#[derive(Deserialize)]
struct Network {
    name: String,
}

fn goals(c: &Competitor) -> u16 {
    c.score.unwrap_or(0.0).max(0.0) as u16
}

fn to_match(g: Game) -> Option<Match> {
    let competition = ids::competition_name(g.competition_id)?;
    let kickoff_utc = chrono::DateTime::parse_from_rfc3339(&g.start_time)
        .ok()?
        .timestamp();
    let status = match g.status_group {
        1 | 2 => MatchStatus::Upcoming,
        3 => {
            let shown = g.game_time_display.trim();
            MatchStatus::Live {
                clock: if shown.is_empty() {
                    g.short_status_text
                } else {
                    shown.to_string()
                },
                home: goals(&g.home_competitor),
                away: goals(&g.away_competitor),
            }
        }
        4 => MatchStatus::Finished,
        _ => return None,
    };
    Some(Match {
        key: format!("s365:{}", g.id),
        competition: competition.to_string(),
        home: g.home_competitor.name,
        away: g.away_competitor.name,
        kickoff_utc,
        status,
        broadcasts: Vec::new(),
    })
}

pub fn parse_day(body: &str) -> Result<Vec<Match>> {
    let day: Day = serde_json::from_str(body).context("365scores day json")?;
    Ok(day
        .games
        .into_iter()
        .filter_map(|v| serde_json::from_value::<Game>(v).ok())
        .filter_map(to_match)
        .collect())
}

pub fn parse_tv(body: &str, country: &str) -> Result<Vec<Broadcast>> {
    let root: TvRoot = serde_json::from_str(body).context("365scores tv json")?;
    let mut out: Vec<Broadcast> = Vec::new();
    for v in root.game.tv_networks {
        let Ok(n) = serde_json::from_value::<Network>(v) else {
            continue;
        };
        let channel = n.name.trim();
        if channel.is_empty() || out.iter().any(|b| b.channel == channel) {
            continue;
        }
        out.push(Broadcast {
            channel: channel.to_string(),
            country: country.to_ascii_uppercase(),
            lang: lang::for_channel(channel, country),
            provider: NAME.into(),
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::super::game_id;
    use super::*;

    const DAY_FIXTURE: &str = include_str!("../../../tests/fixtures/sports/s365_day.json");
    const TV_FIXTURE: &str = include_str!("../../../tests/fixtures/sports/s365_tv.json");

    fn game(id: i64, comp: i64, group: i64, score: f64) -> String {
        format!(
            r#"{{"id":{id},"competitionId":{comp},"startTime":"2026-11-16T19:45:00+00:00","statusGroup":{group},
            "shortStatusText":"HT","gameTimeDisplay":" 67' ",
            "homeCompetitor":{{"name":"A","score":{score}}},"awayCompetitor":{{"name":"B","score":{score}}}}}"#
        )
    }

    fn day(games: &[String]) -> String {
        format!(r#"{{"games":[{}]}}"#, games.join(","))
    }

    #[test]
    fn status_mapping() {
        let body = day(&[
            game(1, 7, 2, -1.0),
            game(2, 7, 3, 2.0),
            game(3, 7, 4, 1.0),
            game(4, 7, 9, 0.0),
        ]);
        let ms = parse_day(&body).unwrap();
        assert_eq!(ms.len(), 3);
        assert_eq!(ms[0].status, MatchStatus::Upcoming);
        assert_eq!(
            ms[1].status,
            MatchStatus::Live {
                clock: "67'".into(),
                home: 2,
                away: 2
            }
        );
        assert_eq!(ms[2].status, MatchStatus::Finished);
        assert_eq!(ms[0].key, "s365:1");
        assert_eq!(ms[0].competition, "Premier League");
    }

    #[test]
    fn live_negative_score_is_zero() {
        let ms = parse_day(&day(&[game(1, 7, 3, -1.0)])).unwrap();
        assert_eq!(
            ms[0].status,
            MatchStatus::Live {
                clock: "67'".into(),
                home: 0,
                away: 0
            }
        );
    }

    #[test]
    fn skips_other_competitions_and_malformed() {
        let body = day(&[
            game(1, 99999, 2, 0.0),
            r#"{"competitionId":7}"#.into(),
            game(3, 7, 2, 0.0),
        ]);
        let ms = parse_day(&body).unwrap();
        assert_eq!(ms.len(), 1);
        assert_eq!(ms[0].key, "s365:3");
    }

    #[test]
    fn missing_games_key_is_error() {
        assert!(parse_day("{}").is_err());
        assert!(parse_tv("{}", "DE").is_err());
    }

    #[test]
    fn tv_dedupes_trims_and_sets_lang() {
        let body = r#"{"game":{"tvNetworks":[{"name":" beIN Sport 1 HD "},{"name":"beIN Sport 1 HD"},{"name":"  "},{"x":1}]}}"#;
        let bs = parse_tv(body, "sa").unwrap();
        assert_eq!(bs.len(), 1);
        assert_eq!(bs[0].channel, "beIN Sport 1 HD");
        assert_eq!(bs[0].country, "SA");
        assert_eq!(bs[0].lang.as_deref(), Some("ar"));
        assert_eq!(bs[0].provider, "365scores");
    }

    #[test]
    fn day_fixture() {
        let ms = parse_day(DAY_FIXTURE).unwrap();
        for m in &ms {
            assert!(m.key.starts_with("s365:"));
            assert!(m.kickoff_utc > 1_700_000_000);
            assert!(!m.home.is_empty() && !m.away.is_empty());
        }
    }

    #[test]
    fn tv_fixture() {
        assert!(!parse_tv(TV_FIXTURE, "DE").unwrap().is_empty());
    }

    #[test]
    fn game_id_from_key() {
        assert_eq!(game_id("s365:42"), Some(42));
        assert_eq!(game_id("livescore:1"), None);
    }
}
