mod client;
mod convert;
mod de;
mod series;
#[cfg(test)]
mod tests;
pub mod urls;

use crate::model::{MediaKind, ParsedPlaylist, Season, Source};
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

pub fn fetch_series(source: &Source, series_key: &str) -> Result<Vec<Season>> {
    let info = Client::new(source)?
        .call(Some("get_series_info"), &[("series_id", series_key)])
        .context("load series info")?;
    Ok(series::parse_seasons(&info))
}
