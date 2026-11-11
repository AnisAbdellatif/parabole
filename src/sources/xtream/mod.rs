mod account;
mod client;
mod convert;
mod de;
mod failover;
mod info;
mod series;
#[cfg(test)]
mod tests;
pub mod urls;

use crate::model::{MediaInfo, MediaKind, ParsedPlaylist, Season, Source};
pub use account::{AccountInfo, Expiry, format_date};
use anyhow::{Context, Result};
use client::Client;

fn credentials(source: &Source) -> Result<(&str, &str)> {
    match (source.username.as_deref(), source.password.as_deref()) {
        (Some(u), Some(p)) => Ok((u, p)),
        _ => anyhow::bail!("source has no credentials"),
    }
}

/// Fetches account info, categories and all streams, strictly one request at a time.
pub fn fetch(source: &Source) -> Result<ParsedPlaylist> {
    failover::run(source, fetch_one)
}

fn fetch_one(source: &Source) -> Result<ParsedPlaylist> {
    let client = Client::new(source)?;
    let account = client.call(None, &[])?;
    convert::check_account(&account)?;

    let mut cats = Vec::new();
    for media in MediaKind::ALL {
        cats.push(client.list(convert::actions(media).0)?);
    }
    let mut items = Vec::new();
    for media in MediaKind::ALL {
        items.push(client.list(convert::actions(media).1)?);
    }

    let mut out = ParsedPlaylist {
        account_json: Some(convert::account_json(&account)),
        ..Default::default()
    };
    for (i, media) in MediaKind::ALL.into_iter().enumerate() {
        convert::convert_media(media, &cats[i], &items[i], &mut out);
    }
    Ok(out)
}

/// Account call only; returns the `account_json` to store.
pub fn check_account(source: &Source) -> Result<String> {
    failover::run(source, |s| {
        let account = Client::new(s)?.call(None, &[])?;
        convert::check_account(&account)?;
        Ok(convert::account_json(&account))
    })
}

pub fn fetch_vod_info(source: &Source, vod_id: &str) -> Result<MediaInfo> {
    failover::run(source, |s| {
        let response = Client::new(s)?
            .call(Some("get_vod_info"), &[("vod_id", vod_id)])
            .context("load movie info")?;
        Ok(info::parse_info(&response))
    })
}

/// Info section and seasons from one `get_series_info` call.
pub fn fetch_series(source: &Source, series_key: &str) -> Result<(MediaInfo, Vec<Season>)> {
    failover::run(source, |s| fetch_series_one(s, series_key))
}

fn fetch_series_one(source: &Source, series_key: &str) -> Result<(MediaInfo, Vec<Season>)> {
    let response = Client::new(source)?
        .call(Some("get_series_info"), &[("series_id", series_key)])
        .context("load series info")?;
    Ok((
        info::parse_info(&response),
        series::parse_seasons(&response),
    ))
}
