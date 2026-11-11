mod attrs;
mod entry;
mod finish;
#[cfg(test)]
mod tests;

use crate::model::ParsedPlaylist;
use anyhow::{Context, Result};
use entry::Pending;
use std::io::BufRead;

pub const UNCATEGORIZED: &str = "Uncategorized";

/// Streaming M3U parser (PLAN.md §5.1).
pub fn parse<R: BufRead>(reader: R) -> Result<ParsedPlaylist> {
    Ok(parse_detect(reader)?.0)
}

/// Like `parse`, also reporting whether the text is an HLS playlist rather than a channel list.
pub fn parse_detect<R: BufRead>(mut reader: R) -> Result<(ParsedPlaylist, bool)> {
    let mut buf = Vec::new();
    let mut drafts = Vec::new();
    let mut epg_url = None;
    let mut is_hls = false;
    let mut pending: Option<Pending> = None;
    loop {
        buf.clear();
        if reader
            .read_until(b'\n', &mut buf)
            .context("read playlist")?
            == 0
        {
            break;
        }
        let text = String::from_utf8_lossy(&buf);
        let line = text.trim().trim_start_matches('\u{feff}').trim();
        if line.is_empty() {
            continue;
        }
        if let Some(rest) = line.strip_prefix("#EXTM3U") {
            let header = attrs::scan(rest);
            epg_url = attrs::get(&header.attrs, "url-tvg")
                .or_else(|| attrs::get(&header.attrs, "x-tvg-url"))
                .map(str::to_string);
        } else if let Some(rest) = line.strip_prefix("#EXTINF:") {
            pending.get_or_insert_default().apply_extinf(rest);
        } else if let Some(group) = line.strip_prefix("#EXTGRP:") {
            pending.get_or_insert_default().set_extgrp(group);
        } else if let Some(opt) = line.strip_prefix("#EXTVLCOPT:") {
            pending.get_or_insert_default().apply_vlcopt(opt);
        } else if let Some(prop) = line.strip_prefix("#KODIPROP:") {
            if prop
                .trim_start()
                .starts_with("inputstream.adaptive.license")
            {
                pending.get_or_insert_default().mark_drm();
            }
        } else if line.starts_with('#') {
            // Tags that only HLS playlists carry; IPTV lists never use them.
            is_hls |= [
                "#EXT-X-STREAM-INF",
                "#EXT-X-TARGETDURATION",
                "#EXT-X-MEDIA-SEQUENCE",
            ]
            .iter()
            .any(|tag| line.starts_with(tag));
        } else {
            drafts.push(pending.take().unwrap_or_default().into_draft(line));
        }
    }
    let (categories, streams) = finish::finish(drafts);
    let playlist = ParsedPlaylist {
        categories,
        streams,
        epg_url,
        account_json: None,
    };
    Ok((playlist, is_hls))
}
