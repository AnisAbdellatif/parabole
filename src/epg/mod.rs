mod fetch;
mod parse;
mod time;
mod window;

use crate::model::Programme;
use anyhow::Result;
use std::collections::HashSet;

pub use window::Window;

/// Downloads and parses every guide URL, keeping programmes of `channels` (lowercased ids) inside `window`.
pub fn fetch_all(
    urls: &[String],
    user_agent: Option<&str>,
    channels: &HashSet<String>,
    window: Window,
) -> Result<Vec<Programme>> {
    let mut parts = Vec::new();
    let mut last_err = None;
    for url in urls {
        let one = fetch::open(url, user_agent).and_then(|r| parse::parse(r, channels, window));
        match one {
            Ok(p) => parts.push(p),
            Err(e) => last_err = Some(e),
        }
    }
    match last_err {
        Some(e) if parts.is_empty() => Err(e.context("guide download failed")),
        _ => Ok(merge(parts)),
    }
}

fn merge(parts: Vec<Vec<Programme>>) -> Vec<Programme> {
    let mut seen = HashSet::new();
    parts
        .into_iter()
        .flatten()
        .filter(|p| seen.insert((p.channel.clone(), p.start)))
        .collect()
}

#[cfg(test)]
mod tests;
