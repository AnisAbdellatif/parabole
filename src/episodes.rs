//! Pure helpers over a series' season list.
use crate::model::{Episode, Season};

pub fn find<'a>(seasons: &'a [Season], id: &str) -> Option<(&'a Season, &'a Episode)> {
    ordered(seasons).into_iter().find(|(_, e)| e.id == id)
}

/// The episode after `id`: next in its season, then the first of the next season.
pub fn next<'a>(seasons: &'a [Season], id: &str) -> Option<(&'a Season, &'a Episode)> {
    let all = ordered(seasons);
    let at = all.iter().position(|(_, e)| e.id == id)?;
    all.get(at + 1).copied()
}

/// "S01E03 – title".
pub fn label(season: &Season, ep: &Episode) -> String {
    match ep.episode_num {
        Some(n) => format!("S{:02}E{n:02} – {}", season.number, ep.title),
        None => format!("S{:02} – {}", season.number, ep.title),
    }
}

fn ordered(seasons: &[Season]) -> Vec<(&Season, &Episode)> {
    let mut sorted: Vec<&Season> = seasons.iter().collect();
    sorted.sort_by_key(|s| s.number);
    sorted
        .into_iter()
        .flat_map(|s| s.episodes.iter().map(move |e| (s, e)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn season(number: i64, ids: &[&str]) -> Season {
        Season {
            number,
            name: String::new(),
            episodes: ids
                .iter()
                .enumerate()
                .map(|(i, id)| Episode {
                    id: id.to_string(),
                    title: format!("T{id}"),
                    episode_num: Some(i as i64 + 1),
                    container_ext: None,
                })
                .collect(),
        }
    }

    #[test]
    fn next_goes_through_seasons_in_number_order() {
        let seasons = vec![
            season(2, &["c", "d"]),
            season(1, &["a", "b"]),
            season(3, &[]),
        ];
        let id = |cur| next(&seasons, cur).map(|(_, e)| e.id.as_str());
        assert_eq!(id("a"), Some("b"));
        assert_eq!(id("b"), Some("c"));
        assert_eq!(id("c"), Some("d"));
        assert_eq!(id("d"), None);
        assert_eq!(id("zzz"), None);
    }

    #[test]
    fn labels() {
        let seasons = vec![season(1, &["a", "b"])];
        let (s, e) = find(&seasons, "b").unwrap();
        assert_eq!(label(s, e), "S01E02 – Tb");
    }
}
