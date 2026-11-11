//! Provider input: detection, M3U parsing, Xtream API, URL resolution.
//! Owner: sources agent. Public API below is a contract used by the worker/UI.

pub mod detect;
mod http;
pub mod m3u;
pub mod xtream;

use crate::model::{
    MediaKind, ParsedCategory, ParsedPlaylist, ParsedStream, PlayInfo, Season, Source, SourceKind,
};
use anyhow::{Context, Result, bail};
use std::fs::File;
use std::io::BufReader;

pub const DEFAULT_USER_AGENT: &str = "VLC/3.0.21 LibVLC/3.0.21";

/// Download (or read) and parse a whole source. Blocking; call from the worker thread.
pub fn fetch(source: &Source) -> Result<ParsedPlaylist> {
    let (playlist, is_hls) = match source.kind {
        SourceKind::Xtream => return xtream::fetch(source),
        SourceKind::M3uUrl => {
            let client = http::client(source.user_agent.as_deref())?;
            let resp = http::get(&client, &source.url)?;
            m3u::parse_detect(BufReader::new(http::SafeRead(resp)))?
        }
        SourceKind::M3uFile => {
            let file = File::open(&source.url).context("open playlist file")?;
            m3u::parse_detect(BufReader::new(file))?
        }
    };
    Ok(if is_hls {
        single_stream(source)
    } else {
        playlist
    })
}

fn single_stream(source: &Source) -> ParsedPlaylist {
    let group = m3u::UNCATEGORIZED.to_string();
    ParsedPlaylist {
        categories: vec![ParsedCategory {
            media: MediaKind::Live,
            ext_key: group.clone(),
            name: group.clone(),
        }],
        streams: vec![ParsedStream {
            media: MediaKind::Live,
            ext_key: source.url.clone(),
            category_key: Some(group),
            name: source.name.clone(),
            url: Some(source.url.clone()),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Lazily load seasons/episodes of an Xtream series. `series_key` = the stream's ext_key.
pub fn fetch_series(source: &Source, series_key: &str) -> Result<Vec<Season>> {
    xtream::fetch_series(source, series_key)
}

/// Final playable URL for a stream (M3U: stored url; Xtream: built from creds).
pub fn resolve_url(source: &Source, info: &PlayInfo) -> Result<String> {
    match source.kind {
        SourceKind::Xtream => xtream::urls::resolve(source, info),
        _ => info.url.clone().context("stream has no URL"),
    }
}

/// Playable URL for a series episode.
pub fn episode_url(
    source: &Source,
    episode_id: &str,
    container_ext: Option<&str>,
) -> Result<String> {
    if source.kind != SourceKind::Xtream {
        bail!("episodes are only available for Xtream sources");
    }
    xtream::urls::episode(source, episode_id, container_ext)
}
