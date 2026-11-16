use crate::sources::DEFAULT_USER_AGENT;
use crate::sources::http::{SafeRead, get};
use anyhow::{Context, Result, bail};
use flate2::read::MultiGzDecoder;
use std::io::{BufRead, BufReader, Read};
use std::time::Duration;

const MAX_BYTES: u64 = 1 << 30;

/// Opens a guide URL as a decoded (gunzipped if needed) byte stream.
pub fn open(url: &str, user_agent: Option<&str>) -> Result<Box<dyn BufRead + Send>> {
    let parsed = url::Url::parse(url).map_err(|_| anyhow::anyhow!("invalid guide URL"))?;
    if !matches!(parsed.scheme(), "http" | "https") {
        bail!("guide URL must be http or https");
    }
    let client = reqwest::blocking::Client::builder()
        .user_agent(user_agent.unwrap_or(DEFAULT_USER_AGENT))
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(600))
        .build()
        .map_err(reqwest::Error::without_url)
        .context("build http client")?;
    let resp = get(&client, url, user_agent)?;
    decode(SafeRead(resp))
}

pub(super) fn decode(r: impl Read + Send + 'static) -> Result<Box<dyn BufRead + Send>> {
    let mut r = BufReader::new(r);
    let gz = r
        .fill_buf()
        .context("read guide body")?
        .starts_with(&[0x1f, 0x8b]);
    if gz {
        let d = MultiGzDecoder::new(r).take(MAX_BYTES);
        Ok(Box::new(BufReader::new(d)))
    } else {
        Ok(Box::new(BufReader::new(r.take(MAX_BYTES))))
    }
}
