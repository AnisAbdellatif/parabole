use super::DEFAULT_USER_AGENT;
use anyhow::{Context, Result};
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
pub fn get(client: &Client, url: &str) -> Result<Response> {
    client
        .get(url)
        .send()
        .and_then(|r| r.error_for_status())
        .map_err(reqwest::Error::without_url)
        .context("request failed")
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
