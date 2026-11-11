use super::DEFAULT_USER_AGENT;
use anyhow::{Context, Result, bail};
use reqwest::blocking::{Client, Response};
use std::io::{self, Read};
use std::time::Duration;

pub fn client(user_agent: Option<&str>) -> Result<Client> {
    Client::builder()
        .user_agent(user_agent.unwrap_or(DEFAULT_USER_AGENT))
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(reqwest::Error::without_url)
        .context("build http client")
}

/// GET that never leaks the URL (it may contain credentials) into the error chain.
pub fn get(client: &Client, url: &str, user_agent: Option<&str>) -> Result<Response> {
    let resp = client
        .get(url)
        .send()
        .map_err(reqwest::Error::without_url)
        .context("request failed")?;
    if let Some(hint) = blocked_hint(resp.status().as_u16(), user_agent) {
        bail!(hint);
    }
    resp.error_for_status()
        .map_err(reqwest::Error::without_url)
        .context("request failed")
}

/// Cloudflare-style 403/513 answers to unknown agents; suggest a preset when none is set.
fn blocked_hint(status: u16, user_agent: Option<&str>) -> Option<String> {
    let unset = user_agent.is_none_or(|u| u.trim().is_empty());
    (matches!(status, 403 | 513) && unset).then(|| {
        format!("server refused the request (HTTP {status}); try a User agent preset in the source settings")
    })
}

/// Replaces body read errors, whose text may embed the URL, with a generic message.
pub struct SafeRead<R>(pub R);

impl<R: Read> Read for SafeRead<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0
            .read(buf)
            .map_err(|e| io::Error::new(e.kind(), "connection error while reading body"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hint_only_for_blocked_status_without_user_agent() {
        assert!(blocked_hint(403, None).unwrap().contains("preset"));
        assert!(blocked_hint(513, Some(" ")).is_some());
        assert!(blocked_hint(403, Some("TiviMate")).is_none());
        assert!(blocked_hint(404, None).is_none());
    }
}
