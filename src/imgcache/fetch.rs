//! Size-capped image download for the cache workers.
use super::store::MAX_IMAGE_BYTES;
use crate::sources::{DEFAULT_USER_AGENT, http};
use anyhow::{Context, Result, bail};
use reqwest::blocking::Client;
use std::io::Read;
use std::time::Duration;

pub fn client() -> Result<Client> {
    Client::builder()
        .user_agent(DEFAULT_USER_AGENT)
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(reqwest::Error::without_url)
        .context("build image http client")
}

/// Errors never contain the URL (it may carry Xtream credentials).
pub fn download(client: &Client, url: &str) -> Result<Vec<u8>> {
    let resp = http::get(client, url, Some(DEFAULT_USER_AGENT))?;
    if resp.content_length().is_some_and(|n| n > MAX_IMAGE_BYTES) {
        bail!("image too large");
    }
    let is_text = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.trim_start().to_ascii_lowercase().starts_with("text/"));
    if is_text {
        bail!("not an image");
    }
    let mut bytes = Vec::new();
    http::SafeRead(resp)
        .take(MAX_IMAGE_BYTES + 1)
        .read_to_end(&mut bytes)
        .context("read image body")?;
    if bytes.len() as u64 > MAX_IMAGE_BYTES {
        bail!("image too large");
    }
    if bytes.is_empty() {
        bail!("empty image");
    }
    Ok(bytes)
}
