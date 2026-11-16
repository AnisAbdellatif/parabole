use super::CLIENT_ID;
use anyhow::{Context, Result, bail};
use reqwest::StatusCode;
use reqwest::blocking::{Client, Response};
use std::time::Duration;

pub const API: &str = "https://api.trakt.tv";
const MAX_WAIT_S: u64 = 30;

pub fn is_configured() -> bool {
    !CLIENT_ID.is_empty()
}

fn send(client: &Client, url: &str) -> Result<Response> {
    client
        .get(url)
        .header("trakt-api-version", "2")
        .header("trakt-api-key", CLIENT_ID)
        .header("content-type", "application/json")
        .send()
        .map_err(reqwest::Error::without_url)
        .context("Trakt request")
}

pub fn get_json(client: &Client, path_and_query: &str) -> Result<serde_json::Value> {
    if !is_configured() {
        bail!("Trakt is not configured");
    }
    let url = format!("{API}{path_and_query}");
    let mut resp = send(client, &url)?;
    if resp.status() == StatusCode::TOO_MANY_REQUESTS {
        let wait = resp
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.trim().parse::<u64>().ok())
            .unwrap_or(1)
            .min(MAX_WAIT_S);
        std::thread::sleep(Duration::from_secs(wait));
        resp = send(client, &url)?;
    }
    let body = resp
        .error_for_status()
        .and_then(|r| r.text())
        .map_err(reqwest::Error::without_url)
        .context("Trakt response")?;
    serde_json::from_str(&body).context("parse Trakt response")
}
