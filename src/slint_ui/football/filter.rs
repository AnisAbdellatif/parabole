//! Which matches the Football list shows, and in what order: one day at a time, grouped by competition.
use crate::model::{Match, MatchStatus};
use crate::sports::merge;
use chrono::{DateTime, FixedOffset, NaiveDate};

pub const LAST_DAY: i64 = 7;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Status {
    All,
    Live,
    Upcoming,
}

impl Status {
    pub fn from_index(i: i32) -> Self {
        match i {
            1 => Self::Live,
            2 => Self::Upcoming,
            _ => Self::All,
        }
    }

    pub fn index(self) -> i32 {
        match self {
            Self::All => 0,
            Self::Live => 1,
            Self::Upcoming => 2,
        }
    }
}

pub struct Query<'a> {
    pub search: &'a str,
    /// Days after today (local time); ignored while searching.
    pub day: i64,
    pub status: Status,
}

impl Query<'_> {
    pub fn searching(&self) -> bool {
        !self.search.trim().is_empty()
    }
}

fn date(ts: i64, tz: FixedOffset) -> NaiveDate {
    DateTime::from_timestamp(ts, 0)
        .unwrap_or_default()
        .with_timezone(&tz)
        .date_naive()
}

pub fn is_live(m: &Match) -> bool {
    matches!(m.status, MatchStatus::Live { .. })
}

/// Days from today to the match's local kickoff date; live matches always count as today.
pub fn day_of(m: &Match, now: i64, tz: FixedOffset) -> i64 {
    if is_live(m) {
        return 0;
    }
    (date(m.kickoff_utc, tz) - date(now, tz)).num_days()
}

/// Section a match is listed under: its competition, prefixed by its day while searching across days.
pub fn group(m: &Match, searching: bool, now: i64, tz: FixedOffset) -> (i64, String) {
    let day = if searching { day_of(m, now, tz) } else { 0 };
    (day, m.competition.clone())
}

/// Matches to show, competitions kept together in order of their first match (input is sorted live first, then kickoff).
pub fn visible<'a>(matches: &'a [Match], q: &Query, now: i64, tz: FixedOffset) -> Vec<&'a Match> {
    let words: Vec<String> = merge::simplify(q.search)
        .split_whitespace()
        .map(str::to_string)
        .collect();
    let mut shown: Vec<&Match> = matches
        .iter()
        .filter(|m| match q.status {
            Status::All => true,
            Status::Live => is_live(m),
            Status::Upcoming => !is_live(m),
        })
        .filter(|m| q.searching() || q.status == Status::Live || day_of(m, now, tz) == q.day)
        .filter(|m| {
            let hay = merge::simplify(&format!("{} {} {}", m.home, m.away, m.competition));
            words.iter().all(|w| hay.contains(w.as_str()))
        })
        .collect();
    let searching = q.searching();
    let mut order: Vec<(i64, String)> = Vec::new();
    for m in &shown {
        let g = group(m, searching, now, tz);
        if !order.contains(&g) {
            order.push(g);
        }
    }
    shown.sort_by_key(|m| {
        let g = group(m, searching, now, tz);
        order.iter().position(|x| *x == g)
    });
    shown
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    fn tz() -> FixedOffset {
        FixedOffset::east_opt(3600).unwrap()
    }

    fn ts(d: u32, h: u32) -> i64 {
        Utc.with_ymd_and_hms(2026, 11, d, h, 0, 0)
            .unwrap()
            .timestamp()
    }

    fn m(key: &str, comp: &str, home: &str, kickoff: i64, live: bool) -> Match {
        Match {
            key: key.into(),
            competition: comp.into(),
            home: home.into(),
            away: "Real Betis".into(),
            kickoff_utc: kickoff,
            status: if live {
                MatchStatus::Live {
                    clock: "1'".into(),
                    home: 0,
                    away: 0,
                }
            } else {
                MatchStatus::Upcoming
            },
            broadcasts: vec![],
        }
    }

    fn keys(v: Vec<&Match>) -> Vec<&str> {
        v.iter().map(|m| m.key.as_str()).collect()
    }

    fn q(search: &str, day: i64, status: Status) -> Query<'_> {
        Query {
            search,
            day,
            status,
        }
    }

    #[test]
    fn one_day_grouped_by_competition() {
        let now = ts(16, 10);
        let ms = [
            m("a", "Liga", "Atlético Madrid", ts(16, 9), true),
            m("b", "Serie A", "Inter", ts(16, 18), false),
            m("c", "Liga", "Getafe", ts(16, 20), false),
            m("d", "Liga", "Girona", ts(17, 20), false),
        ];
        assert_eq!(
            keys(visible(&ms, &q("", 0, Status::All), now, tz())),
            ["a", "c", "b"]
        );
        assert_eq!(keys(visible(&ms, &q("", 1, Status::All), now, tz())), ["d"]);
        assert_eq!(
            keys(visible(&ms, &q("", 3, Status::Live), now, tz())),
            ["a"]
        );
        assert_eq!(
            keys(visible(&ms, &q("", 0, Status::Upcoming), now, tz())),
            ["b", "c"]
        );
    }

    #[test]
    fn search_spans_days_folds_accents_and_needs_every_word() {
        let now = ts(16, 10);
        let ms = [
            m("a", "Liga", "Atlético Madrid", ts(16, 18), false),
            m("d", "Liga", "Atletico Madrid", ts(18, 20), false),
        ];
        assert_eq!(
            keys(visible(
                &ms,
                &q("ATLETICO betis", 0, Status::All),
                now,
                tz()
            )),
            ["a", "d"]
        );
        assert!(visible(&ms, &q("atletico barcelona", 0, Status::All), now, tz()).is_empty());
    }
}
