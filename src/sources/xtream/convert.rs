use super::de::{opt_i64, opt_string, string_list};
use crate::model::{MediaKind, ParsedCategory, ParsedPlaylist, ParsedStream};
use crate::sources::m3u::UNCATEGORIZED;
use anyhow::{Result, bail};
use serde_json::{Map, Value};
use std::collections::HashSet;

const UNCATEGORIZED_KEY: &str = "uncategorized";

pub fn actions(media: MediaKind) -> (&'static str, &'static str) {
    match media {
        MediaKind::Live => ("get_live_categories", "get_live_streams"),
        MediaKind::Movie => ("get_vod_categories", "get_vod_streams"),
        MediaKind::Series => ("get_series_categories", "get_series"),
    }
}

pub fn check_account(account: &Value) -> Result<()> {
    let Some(user_info) = account.get("user_info").filter(|u| u.is_object()) else {
        bail!("unexpected account response");
    };
    if let Some(auth) = user_info.get("auth")
        && !super::de::bool01(Some(auth))
    {
        bail!("invalid credentials");
    }
    Ok(())
}

pub fn account_json(account: &Value) -> String {
    serde_json::json!({
        "user_info": account.get("user_info"),
        "server_info": account.get("server_info"),
    })
    .to_string()
}

/// Appends one media kind's categories and streams; malformed items are skipped.
pub fn convert_media(media: MediaKind, cats: &[Value], items: &[Value], out: &mut ParsedPlaylist) {
    let mut known = HashSet::new();
    for cat in cats {
        let Some(id) = opt_string(cat.get("category_id")) else {
            continue;
        };
        if known.insert(id.clone()) {
            let name = opt_string(cat.get("category_name")).unwrap_or_else(|| id.clone());
            out.categories.push(ParsedCategory {
                media,
                ext_key: id,
                name,
            });
        }
    }
    let mut seen = HashSet::new();
    let mut needs_fallback = false;
    for item in items {
        let Some(mut stream) = convert_stream(media, item) else {
            continue;
        };
        if !seen.insert(stream.ext_key.clone()) {
            continue;
        }
        if !stream
            .category_key
            .as_ref()
            .is_some_and(|k| known.contains(k))
        {
            stream.category_key = Some(UNCATEGORIZED_KEY.to_string());
            needs_fallback = true;
        }
        out.streams.push(stream);
    }
    if needs_fallback {
        out.categories.push(ParsedCategory {
            media,
            ext_key: UNCATEGORIZED_KEY.to_string(),
            name: UNCATEGORIZED.to_string(),
        });
    }
}

fn convert_stream(media: MediaKind, item: &Value) -> Option<ParsedStream> {
    if !item.is_object() {
        return None;
    }
    let is_series = media == MediaKind::Series;
    let ext_key = opt_string(item.get(if is_series { "series_id" } else { "stream_id" }))?;
    let name = opt_string(item.get("name")).unwrap_or_else(|| ext_key.clone());
    let logo = opt_string(item.get(if is_series { "cover" } else { "stream_icon" }));

    let mut extra = Map::new();
    for key in ["rating", "added", "plot"] {
        if let Some(v) = opt_string(item.get(key)) {
            extra.insert(key.to_string(), Value::String(v));
        }
    }
    let backdrops = string_list(item.get("backdrop_path"));
    if !backdrops.is_empty() {
        extra.insert("backdrop_path".to_string(), backdrops.into());
    }

    Some(ParsedStream {
        media,
        ext_key,
        category_key: opt_string(item.get("category_id")),
        name,
        num: opt_i64(item.get("num")),
        logo,
        container_ext: opt_string(item.get("container_extension")),
        epg_id: opt_string(item.get("epg_channel_id")),
        archive_days: opt_i64(item.get("tv_archive_duration")).filter(|d| *d > 0),
        extra_json: (!extra.is_empty()).then(|| Value::Object(extra).to_string()),
        ..Default::default()
    })
}
