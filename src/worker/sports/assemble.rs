use super::{AHEAD_DAYS, PAST_S, day_key, days, tv_key};
use crate::db::Db;
use crate::model::{Broadcast, Match, MatchStatus};
use crate::sports::registry::{Sport, TvSource};
use crate::sports::{lang, merge};

fn read<T: serde::de::DeserializeOwned>(db: &Db, key: &str, schema: i64) -> Option<Vec<T>> {
    let json = db
        .sports_cache_get(key, schema, i64::MAX, 0)
        .ok()
        .flatten()?;
    serde_json::from_str(&json).ok()
}

fn add(m: &mut Match, list: Vec<Broadcast>) {
    for b in list {
        let dup = m.broadcasts.iter().any(|x| {
            x.country == b.country && x.channel.to_lowercase() == b.channel.to_lowercase()
        });
        if !dup {
            m.broadcasts.push(b);
        }
    }
}

/// The list the tab shows, built from cache only (any age): the first source's matches,
/// their cached channels for the user's countries, plus other sources' channels paired by `merge::attach`.
pub(super) fn assemble(db: &Db, sport: &Sport, langs: &[String], now: i64) -> Vec<Match> {
    let mut base: Vec<Match> = Vec::new();
    let mut extra: Vec<Match> = Vec::new();
    for (i, src) in sport.days.iter().enumerate() {
        let out = if i == 0 { &mut base } else { &mut extra };
        for date in days(now) {
            if let Some(v) = read::<Match>(db, &day_key(sport.id, src.provider, date), src.schema) {
                out.extend(v);
            }
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
    let schema = sport.days[0].schema;
    for m in &mut base {
        match &sport.tv {
            Some(TvSource::PerCountry(_)) => {
                for country in &countries {
                    let key = tv_key(sport.id, &m.key, Some(country));
                    if let Some(list) = read::<Broadcast>(db, &key, schema) {
                        add(m, list);
                    }
                }
            }
            Some(TvSource::AllCountries(_)) => {
                let key = tv_key(sport.id, &m.key, None);
                if let Some(mut list) = read::<Broadcast>(db, &key, schema) {
                    list.retain(|b| countries.iter().any(|c| c.eq_ignore_ascii_case(&b.country)));
                    add(m, list);
                }
            }
            None => {}
        }
    }
    merge::attach(&mut base, &extra);
    base.sort_by(|a, b| {
        let live = |m: &Match| !matches!(m.status, MatchStatus::Live { .. });
        (live(a), a.kickoff_utc, &a.key).cmp(&(live(b), b.kickoff_utc, &b.key))
    });
    base
}
