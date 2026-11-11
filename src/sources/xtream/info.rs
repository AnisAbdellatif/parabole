use super::de::{opt_string, string_list};
use crate::model::MediaInfo;
use serde_json::Value;

/// Reads the `info` section of `get_vod_info` / `get_series_info`.
/// Panels send `info` as `[]` when empty and vary key names and value types.
pub fn parse_info(response: &Value) -> MediaInfo {
    let Some(info) = response.get("info").filter(|i| i.is_object()) else {
        return MediaInfo::default();
    };
    let text = |keys: &[&str]| {
        keys.iter()
            .map(|k| string_list(info.get(*k)))
            .find(|v| !v.is_empty())
            .map(|v| v.join(", "))
    };
    MediaInfo {
        plot: text(&["plot", "description"]),
        cast: text(&["cast", "actors"]),
        director: text(&["director"]),
        genre: text(&["genre"]),
        release_date: text(&["releasedate", "releaseDate", "release_date"]),
        duration: text(&["duration", "episode_run_time"]),
        rating: text(&["rating", "rating_5based"]),
        cover: ["movie_image", "cover_big", "cover"]
            .iter()
            .find_map(|k| opt_string(info.get(*k))),
        backdrop: string_list(info.get("backdrop_path")).into_iter().next(),
    }
}
