use super::de::{opt_i64, opt_string};
use crate::model::{Episode, Season};
use serde_json::Value;
use std::collections::BTreeMap;

/// Converts a `get_series_info` response into sorted seasons that have episodes.
pub fn parse_seasons(info: &Value) -> Vec<Season> {
    let mut seasons: BTreeMap<i64, Season> = BTreeMap::new();
    if let Some(Value::Array(listed)) = info.get("seasons") {
        for s in listed {
            if let Some(number) = opt_i64(s.get("season_number")) {
                let name = opt_string(s.get("name")).unwrap_or_else(|| format!("Season {number}"));
                seasons.insert(
                    number,
                    Season {
                        number,
                        name,
                        episodes: Vec::new(),
                    },
                );
            }
        }
    }
    let mut found = Vec::new();
    if let Some(episodes) = info.get("episodes") {
        collect(episodes, None, None, &mut found);
    }
    for (number, episode) in found {
        seasons
            .entry(number)
            .or_insert_with(|| Season {
                number,
                name: format!("Season {number}"),
                episodes: Vec::new(),
            })
            .episodes
            .push(episode);
    }
    let mut out: Vec<Season> = seasons
        .into_values()
        .filter(|s| !s.episodes.is_empty())
        .collect();
    for season in &mut out {
        season
            .episodes
            .sort_by_key(|e| e.episode_num.unwrap_or(i64::MAX));
    }
    out
}

/// Walks `episodes`, which is an object keyed by season, an array of arrays, or a flat array.
fn collect(
    v: &Value,
    key_season: Option<i64>,
    fallback: Option<i64>,
    out: &mut Vec<(i64, Episode)>,
) {
    match v {
        Value::Array(items) => {
            for (i, item) in items.iter().enumerate() {
                let fallback = if item.is_array() {
                    Some(i as i64 + 1)
                } else {
                    fallback
                };
                collect(item, key_season, fallback, out);
            }
        }
        Value::Object(map) if map.contains_key("id") => {
            let number = key_season
                .or(opt_i64(v.get("season")))
                .or(fallback)
                .unwrap_or(0);
            if let Some(episode) = episode(v) {
                out.push((number, episode));
            }
        }
        Value::Object(map) => {
            for (key, value) in map {
                collect(value, key.trim().parse().ok().or(key_season), fallback, out);
            }
        }
        _ => {}
    }
}

fn episode(v: &Value) -> Option<Episode> {
    let id = opt_string(v.get("id"))?;
    let episode_num = opt_i64(v.get("episode_num"));
    let title = opt_string(v.get("title"))
        .unwrap_or_else(|| format!("Episode {}", episode_num.unwrap_or_default()));
    Some(Episode {
        id,
        title,
        episode_num,
        container_ext: opt_string(v.get("container_extension")),
    })
}
