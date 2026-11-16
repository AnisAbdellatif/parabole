use super::{AHEAD_DAYS, PAST_S, day_key, days, tv_key};
use crate::db::Db;
use crate::model::{Broadcast, Match, MatchStatus};
use crate::sports::{lang, livescore, merge, s365};

fn read<T: serde::de::DeserializeOwned>(db: &Db, key: &str, schema: i64) -> Option<Vec<T>> {
    let json = db
        .sports_cache_get(key, schema, i64::MAX, 0)
        .ok()
        .flatten()?;
    serde_json::from_str(&json).ok()
}

/// The list the tab shows, built from cache only (any age): allowlisted 365scores matches,
/// their cached channels for the user's countries, plus LiveScore channels paired by `merge::attach`.
pub(super) fn assemble(db: &Db, langs: &[String], now: i64) -> Vec<Match> {
    let mut base: Vec<Match> = Vec::new();
    let mut extra: Vec<Match> = Vec::new();
    for date in days(now) {
        if let Some(v) = read::<Match>(db, &day_key(s365::NAME, date), s365::SCHEMA) {
            base.extend(v);
        }
        if let Some(v) = read::<Match>(db, &day_key(livescore::NAME, date), livescore::SCHEMA) {
            extra.extend(v);
        }
    }
    base.retain(|m| match m.status {
        MatchStatus::Live { .. } => true,
        MatchStatus::Upcoming => {
            m.kickoff_utc >= now - PAST_S && m.kickoff_utc <= now + AHEAD_DAYS * 86400
        }
        MatchStatus::Finished => false,
    });
    let mut seen = std::collections::HashSet::new();
    base.retain(|m| seen.insert(m.key.clone()));

    let countries = lang::countries_for(langs);
    for m in &mut base {
        let Some(id) = s365::game_id(&m.key) else {
            continue;
        };
        for country in &countries {
            let Some(list) = read::<Broadcast>(db, &tv_key(id, country), s365::SCHEMA) else {
                continue;
            };
            for b in list {
                let dup = m.broadcasts.iter().any(|x| {
                    x.country == b.country && x.channel.to_lowercase() == b.channel.to_lowercase()
                });
                if !dup {
                    m.broadcasts.push(b);
                }
            }
        }
    }
    merge::attach(&mut base, &extra);
    base.sort_by(|a, b| {
        let live = |m: &Match| !matches!(m.status, MatchStatus::Live { .. });
        (live(a), a.kickoff_utc, &a.key).cmp(&(live(b), b.kickoff_utc, &b.key))
    });
    base
}
