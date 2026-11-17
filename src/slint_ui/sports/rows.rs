//! Text for the Sports tab: pure formatting of matches, no Slint types.
use super::filter;
use crate::model::{Broadcast, Match, MatchStatus, ProviderStatus};
use crate::sports::lang;
use chrono::{DateTime, TimeZone};
use std::collections::BTreeSet;

pub struct RowText {
    pub key: String,
    pub section: String,
    pub time: String,
    pub live: bool,
    pub home: String,
    pub away: String,
    pub home_score: String,
    pub away_score: String,
    pub langs: String,
}

pub struct ChannelText {
    pub header: bool,
    pub text: String,
    pub channel: String,
}

const CHIPS_SHOWN: usize = 4;

fn local<Tz: TimeZone>(ts: i64, tz: &Tz) -> Option<DateTime<Tz>> {
    DateTime::from_timestamp(ts, 0).map(|t| t.with_timezone(tz))
}

fn fmt<Tz: TimeZone>(ts: i64, tz: &Tz, pattern: &str) -> String
where
    Tz::Offset: std::fmt::Display,
{
    local(ts, tz).map_or_else(String::new, |t| t.format(pattern).to_string())
}

fn teams(m: &Match) -> String {
    if m.away.is_empty() {
        m.home.clone()
    } else {
        format!("{} – {}", m.home, m.away)
    }
}

/// "Today", "Tomorrow" or "Wed 18 Nov" for `day` days after today.
pub fn day_label<Tz: TimeZone>(day: i64, now: i64, tz: &Tz) -> String
where
    Tz::Offset: std::fmt::Display,
{
    match day {
        0 => "Today".into(),
        1 => "Tomorrow".into(),
        // calendar days, not 86400 s steps: a DST day is 23 or 25 h long
        _ => tz
            .timestamp_opt(now, 0)
            .single()
            .zip(u64::try_from(day).ok())
            .and_then(|(t, d)| t.date_naive().checked_add_days(chrono::Days::new(d)))
            .map(|d| d.format("%a %-d %b").to_string())
            .unwrap_or_default(),
    }
}

/// `matches` in display order; a section header starts each competition (and day, while searching).
pub fn rows<Tz: TimeZone>(
    matches: &[&Match],
    langs: &[String],
    searching: bool,
    now: i64,
    tz: &Tz,
) -> Vec<RowText>
where
    Tz::Offset: std::fmt::Display,
{
    let mut prev = None;
    matches
        .iter()
        .map(|m| {
            let g = filter::group(m, searching, now, tz);
            let section = if prev.as_ref() == Some(&g) {
                String::new()
            } else if searching {
                format!("{} · {}", day_label(g.0, now, tz), g.1)
            } else {
                g.1.clone()
            };
            prev = Some(g);
            let (time, live, home_score, away_score) = match &m.status {
                MatchStatus::Live { clock, home, away } => {
                    (clock.clone(), true, home.clone(), away.clone())
                }
                _ => (
                    fmt(m.kickoff_utc, tz, "%H:%M"),
                    false,
                    String::new(),
                    String::new(),
                ),
            };
            RowText {
                key: m.key.clone(),
                section,
                time,
                live,
                home: m.home.clone(),
                away: m.away.clone(),
                home_score,
                away_score,
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

pub fn details<Tz: TimeZone>(
    m: &Match,
    langs: &[String],
    tz: &Tz,
) -> (String, String, Vec<ChannelText>)
where
    Tz::Offset: std::fmt::Display,
{
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
        // One line per channel, listing every country it airs in (sorted, so same channels are adjacent).
        let mut lines: Vec<(&str, Vec<&str>)> = Vec::new();
        for x in group {
            match lines.last_mut() {
                Some((ch, cs)) if ch.to_lowercase() == x.channel.to_lowercase() => {
                    cs.push(&x.country)
                }
                _ => lines.push((&x.channel, vec![&x.country])),
            }
        }
        out.extend(lines.into_iter().map(|(ch, cs)| ChannelText {
            header: false,
            text: format!("{ch} ({})", cs.join(", ")),
            channel: ch.to_string(),
        }));
    }
    out
}

pub fn status<Tz: TimeZone>(
    providers: &[ProviderStatus],
    loading: bool,
    updated_at: Option<i64>,
    tz: &Tz,
) -> String
where
    Tz::Offset: std::fmt::Display,
{
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
        "Loading…"
    } else {
        "No events in the selected competitions on this day."
    }
}

#[cfg(test)]
#[path = "rows_tests.rs"]
mod tests;
