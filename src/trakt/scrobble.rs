//! Request bodies for scrobbling and watch history; pure, no network.
use serde_json::{Value, json};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Start,
    Pause,
    Stop,
}

impl Action {
    pub fn path(self) -> &'static str {
        match self {
            Action::Start => "/scrobble/start",
            Action::Pause => "/scrobble/pause",
            Action::Stop => "/scrobble/stop",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Ids {
    Imdb(String),
    Trakt(i64),
}

impl Ids {
    /// Reads a stored link: `tt…`, `trakt:movie:123` or `trakt:show:123`.
    pub fn from_meta_id(id: &str) -> Option<Ids> {
        if id.starts_with("tt") {
            return Some(Ids::Imdb(id.to_string()));
        }
        let mut parts = id.split(':');
        match (parts.next(), parts.next(), parts.next(), parts.next()) {
            (Some("trakt"), Some("movie" | "show"), Some(n), None) => {
                n.parse().ok().map(Ids::Trakt)
            }
            _ => None,
        }
    }

    fn json(&self) -> Value {
        match self {
            Ids::Imdb(id) => json!({ "imdb": id }),
            Ids::Trakt(id) => json!({ "trakt": id }),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Item {
    Movie(Ids),
    Episode { show: Ids, season: i64, number: i64 },
}

pub fn scrobble_body(item: &Item, progress: f32) -> Value {
    let progress = progress.clamp(0.0, 100.0);
    match item {
        Item::Movie(ids) => json!({ "movie": { "ids": ids.json() }, "progress": progress }),
        Item::Episode {
            show,
            season,
            number,
        } => json!({
            "show": { "ids": show.json() },
            "episode": { "season": season, "number": number },
            "progress": progress,
        }),
    }
}

/// Body of `/sync/history` (with `watched_at`) or `/sync/history/remove` (without).
pub fn history_body(item: &Item, watched_at: Option<&str>) -> Value {
    let stamp = |mut v: Value| {
        if let Some(at) = watched_at {
            v["watched_at"] = json!(at);
        }
        v
    };
    match item {
        Item::Movie(ids) => json!({ "movies": [stamp(json!({ "ids": ids.json() }))] }),
        Item::Episode {
            show,
            season,
            number,
        } => json!({ "shows": [{
            "ids": show.json(),
            "seasons": [{ "number": season, "episodes": [stamp(json!({ "number": number }))] }],
        }] }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn movie() -> Item {
        Item::Movie(Ids::Imdb("tt0113277".into()))
    }

    fn episode() -> Item {
        Item::Episode {
            show: Ids::Trakt(1390),
            season: 1,
            number: 2,
        }
    }

    #[test]
    fn meta_ids() {
        assert_eq!(
            Ids::from_meta_id("tt0113277"),
            Some(Ids::Imdb("tt0113277".into()))
        );
        assert_eq!(Ids::from_meta_id("trakt:movie:123"), Some(Ids::Trakt(123)));
        assert_eq!(Ids::from_meta_id("trakt:show:9"), Some(Ids::Trakt(9)));
        assert_eq!(Ids::from_meta_id("trakt:person:9"), None);
        assert_eq!(Ids::from_meta_id("trakt:show:x"), None);
        assert_eq!(Ids::from_meta_id("trakt:show:1:2"), None);
        assert_eq!(Ids::from_meta_id(""), None);
    }

    #[test]
    fn scrobble_movie_and_episode() {
        assert_eq!(
            scrobble_body(&movie(), 42.5),
            json!({"movie": {"ids": {"imdb": "tt0113277"}}, "progress": 42.5})
        );
        assert_eq!(
            scrobble_body(&episode(), 10.0),
            json!({
                "show": {"ids": {"trakt": 1390}},
                "episode": {"season": 1, "number": 2},
                "progress": 10.0
            })
        );
    }

    #[test]
    fn progress_is_clamped() {
        assert_eq!(scrobble_body(&movie(), 140.0)["progress"], json!(100.0));
        assert_eq!(scrobble_body(&movie(), -3.0)["progress"], json!(0.0));
    }

    #[test]
    fn history_bodies() {
        let at = "2026-11-17T10:00:00Z";
        assert_eq!(
            history_body(&movie(), Some(at)),
            json!({"movies": [{"ids": {"imdb": "tt0113277"}, "watched_at": at}]})
        );
        assert_eq!(
            history_body(&episode(), Some(at)),
            json!({"shows": [{
                "ids": {"trakt": 1390},
                "seasons": [{"number": 1, "episodes": [{"number": 2, "watched_at": at}]}]
            }]})
        );
        assert_eq!(
            history_body(&episode(), None),
            json!({"shows": [{
                "ids": {"trakt": 1390},
                "seasons": [{"number": 1, "episodes": [{"number": 2}]}]
            }]})
        );
    }
}
