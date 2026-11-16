//! Text for the Football tab: pure formatting of matches, no Slint types.
use crate::model::{Broadcast, Match, MatchStatus, ProviderStatus};
use crate::sports::lang;
use chrono::{DateTime, Duration, FixedOffset};
use std::collections::BTreeSet;

pub struct RowText {
    pub key: String,
    pub section: String,
    pub time: String,
    pub live: bool,
    pub teams: String,
    pub score: String,
    pub competition: String,
    pub langs: String,
}

pub struct ChannelText {
    pub header: bool,
    pub text: String,
    pub channel: String,
}

const CHIPS_SHOWN: usize = 4;

fn local(ts: i64, tz: FixedOffset) -> Option<DateTime<FixedOffset>> {
    DateTime::from_timestamp(ts, 0).map(|t| t.with_timezone(&tz))
}

fn fmt(ts: i64, tz: FixedOffset, pattern: &str) -> String {
    local(ts, tz).map_or_else(String::new, |t| t.format(pattern).to_string())
}

fn teams(m: &Match) -> String {
    format!("{} – {}", m.home, m.away)
}

fn group_label(m: &Match, now: i64, tz: FixedOffset) -> String {
    if matches!(m.status, MatchStatus::Live { .. }) {
        return "LIVE".into();
    }
    let (Some(k), Some(n)) = (local(m.kickoff_utc, tz), local(now, tz)) else {
        return String::new();
    };
    let (kd, nd) = (k.date_naive(), n.date_naive());
    if kd == nd {
        "TODAY".into()
    } else if kd == nd + Duration::days(1) {
        "TOMORROW".into()
    } else {
        k.format("%a %-d %b").to_string().to_uppercase()
    }
}

pub fn rows(matches: &[Match], langs: &[String], now: i64, tz: FixedOffset) -> Vec<RowText> {
    let mut prev = String::new();
    matches
        .iter()
        .map(|m| {
            let label = group_label(m, now, tz);
            let section = if label != prev {
                prev.clone_from(&label);
                label
            } else {
                String::new()
            };
            let (time, live, score) = match &m.status {
                MatchStatus::Live { clock, home, away } => {
                    (clock.clone(), true, format!("{home}–{away}"))
                }
                _ => (fmt(m.kickoff_utc, tz, "%H:%M"), false, String::new()),
            };
            RowText {
                key: m.key.clone(),
                section,
                time,
                live,
                teams: teams(m),
                score,
                competition: m.competition.clone(),
                langs: chips(&m.broadcasts, langs),
            }
        })
        .collect()
}

fn chips(b: &[Broadcast], user: &[String]) -> String {
    let present: BTreeSet<&str> = b.iter().filter_map(|x| x.lang.as_deref()).collect();
    let mut ordered: Vec<&str> = user
        .iter()
        .map(String::as_str)
        .filter(|c| present.contains(c))
        .collect();
    let rest: Vec<&str> = present
        .iter()
        .copied()
        .filter(|c| !ordered.contains(c))
        .collect();
    ordered.extend(rest);
    let shown: Vec<String> = ordered
        .iter()
        .take(CHIPS_SHOWN)
        .map(|c| c.to_uppercase())
        .collect();
    let mut out = shown.join(" ");
    if ordered.len() > CHIPS_SHOWN {
        out.push_str(&format!(" +{}", ordered.len() - CHIPS_SHOWN));
    }
    out
}

pub fn details(m: &Match, langs: &[String], tz: FixedOffset) -> (String, String, Vec<ChannelText>) {
    let sub = match &m.status {
        MatchStatus::Live { clock, home, away } => {
            format!("{} · Live {clock} · {home}–{away}", m.competition)
        }
        MatchStatus::Finished => format!("{} · Finished", m.competition),
        MatchStatus::Upcoming => format!(
            "{} · {}",
            m.competition,
            fmt(m.kickoff_utc, tz, "%a %-d %b, %H:%M")
        ),
    };
    (teams(m), sub, channels(&m.broadcasts, langs))
}

fn channels(b: &[Broadcast], user: &[String]) -> Vec<ChannelText> {
    let mut seen = BTreeSet::new();
    let uniq: Vec<&Broadcast> = b
        .iter()
        .filter(|x| seen.insert((x.channel.to_lowercase(), x.country.clone())))
        .collect();
    let mut codes: Vec<Option<&str>> = user
        .iter()
        .map(|c| Some(c.as_str()))
        .filter(|c| uniq.iter().any(|x| x.lang.as_deref() == *c))
        .collect();
    let mut others: Vec<&str> = uniq
        .iter()
        .filter_map(|x| x.lang.as_deref())
        .filter(|c| !user.iter().any(|u| u == c))
        .collect();
    others.sort_by_key(|c| lang::name(c));
    others.dedup();
    codes.extend(others.into_iter().map(Some));
    if uniq.iter().any(|x| x.lang.is_none()) {
        codes.push(None);
    }
    let mut out = Vec::new();
    for code in codes {
        let mut group: Vec<&&Broadcast> =
            uniq.iter().filter(|x| x.lang.as_deref() == code).collect();
        group.sort_by_key(|x| (x.channel.to_lowercase(), x.country.clone()));
        out.push(ChannelText {
            header: true,
            text: code.map_or_else(|| "Other / multilingual".into(), |c| lang::name(c).into()),
            channel: String::new(),
        });
        out.extend(group.into_iter().map(|x| ChannelText {
            header: false,
            text: format!("{} ({})", x.channel, x.country),
            channel: x.channel.clone(),
        }));
    }
    out
}

pub fn status(
    providers: &[ProviderStatus],
    loading: bool,
    updated_at: Option<i64>,
    tz: FixedOffset,
) -> String {
    let mut parts = Vec::new();
    if loading {
        parts.push("Updating…".to_string());
    } else if let Some(t) = updated_at {
        parts.push(format!("Updated {}", fmt(t, tz, "%H:%M")));
    }
    parts.extend(
        providers
            .iter()
            .filter(|p| !p.ok)
            .map(|p| format!("{} unavailable", p.name)),
    );
    parts.join(" · ")
}

pub fn empty_text(loading: bool) -> &'static str {
    if loading {
        "Loading matches…"
    } else {
        "No upcoming matches in the selected competitions."
    }
}

#[cfg(test)]
#[path = "rows_tests.rs"]
mod tests;
