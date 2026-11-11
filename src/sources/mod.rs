//! Provider input: detection, M3U parsing, Xtream API, URL resolution.
//! Owner: sources agent. Public API below is a contract used by the worker/UI.

pub mod detect;
pub mod m3u;
pub mod xtream;

use crate::model::{ParsedPlaylist, PlayInfo, Season, Source};
use anyhow::Result;

/// Download (or read) and parse a whole source. Blocking; call from the worker thread.
pub fn fetch(source: &Source) -> Result<ParsedPlaylist> {
    let _ = source;
    todo!()
}

/// Lazily load seasons/episodes of an Xtream series. `series_key` = the stream's ext_key.
pub fn fetch_series(source: &Source, series_key: &str) -> Result<Vec<Season>> {
    let _ = (source, series_key);
    todo!()
}

/// Final playable URL for a stream (M3U: stored url; Xtream: built from creds).
pub fn resolve_url(source: &Source, info: &PlayInfo) -> Result<String> {
    let _ = (source, info);
    todo!()
}

/// Playable URL for a series episode.
pub fn episode_url(source: &Source, episode_id: &str, container_ext: Option<&str>) -> Result<String> {
    let _ = (source, episode_id, container_ext);
    todo!()
}
