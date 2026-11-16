use super::CLIENT_ID;
use anyhow::{Context, Result, bail};
use reqwest::blocking::{Client, Response};
use reqwest::{Method, StatusCode};
use serde_json::Value;
use std::time::Duration;

pub const API: &str = "https://api.trakt.tv";
const MAX_WAIT_S: u64 = 30;

pub fn is_configured() -> bool {
    !CLIENT_ID.is_empty()
}

fn send(
    client: &Client,
    method: &Method,
    url: &str,
    body: Option<&Value>,
    token: Option<&str>,
) -> Result<Response> {
    let mut req = client
        .request(method.clone(), url)
        .header("trakt-api-version", "2")
        .header("trakt-api-key", CLIENT_ID)
        .header("content-type", "application/json");
    if let Some(t) = token {
        req = req.bearer_auth(t);
    }
    if let Some(b) = body {
        req = req.body(b.to_string());
    }
    req.send()
        .map_err(reqwest::Error::without_url)
        .context("Trakt request")
}

/// Sends once, and once more after the `Retry-After` of a 429.
fn call(
    client: &Client,
    method: Method,
    path_and_query: &str,
    body: Option<&Value>,
    token: Option<&str>,
) -> Result<Response> {
    if !is_configured() {
        bail!("Trakt is not configured");
    }
    let url = format!("{API}{path_and_query}");
    let resp = send(client, &method, &url, body, token)?;
    if resp.status() != StatusCode::TOO_MANY_REQUESTS {
        return Ok(resp);
    }
    let wait = resp
        .headers()
        .get("retry-after")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.trim().parse::<u64>().ok())
        .unwrap_or(1)
        .min(MAX_WAIT_S);
    std::thread::sleep(Duration::from_secs(wait));
    send(client, &method, &url, body, token)
}

fn json_of(resp: Response) -> Result<Value> {
    let body = resp
        .error_for_status()
        .and_then(|r| r.text())
        .map_err(reqwest::Error::without_url)
        .context("Trakt response")?;
    serde_json::from_str(&body).context("parse Trakt response")
}

pub fn get_json(client: &Client, path_and_query: &str) -> Result<Value> {
    json_of(call(client, Method::GET, path_and_query, None, None)?)
}

/// GET with the user's access token.
pub fn get_authed(client: &Client, path_and_query: &str, token: &str) -> Result<Value> {
    json_of(call(
        client,
        Method::GET,
        path_and_query,
        None,
        Some(token),
    )?)
}

/// POST returning the status and the parsed body (`Null` when empty); 4xx/5xx are not errors here.
pub fn post_json(
    client: &Client,
    path: &str,
    body: &Value,
    token: Option<&str>,
) -> Result<(u16, Value)> {
    let resp = call(client, Method::POST, path, Some(body), token)?;
    let status = resp.status().as_u16();
    let text = resp
        .text()
        .map_err(reqwest::Error::without_url)
        .context("Trakt response")?;
    Ok((status, serde_json::from_str(&text).unwrap_or(Value::Null)))
}
