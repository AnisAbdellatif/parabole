mod fetch;
pub mod matching;
mod parse;
mod time;
mod window;

use crate::model::{GuideChannel, Programme};
use anyhow::Result;
use std::collections::{HashMap, HashSet};

pub use window::Window;

#[derive(Debug, Default)]
pub struct ParsedGuide {
    pub channels: Vec<GuideChannel>,
    pub programmes: Vec<Programme>,
}

/// Downloads and parses every guide URL; `select` picks the channels to keep per guide (see `parse::parse`).
pub fn fetch_all(
    urls: &[String],
    user_agent: Option<&str>,
    window: Window,
    select: &mut dyn FnMut(&[GuideChannel]) -> HashSet<String>,
) -> Result<ParsedGuide> {
    let mut parts = Vec::new();
    let mut last_err = None;
    for url in urls {
        let one = fetch::open(url, user_agent).and_then(|r| parse::parse(r, window, select));
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

fn merge(parts: Vec<ParsedGuide>) -> ParsedGuide {
    let mut out = ParsedGuide::default();
    let mut index: HashMap<String, usize> = HashMap::new();
    let mut seen = HashSet::new();
    for part in parts {
        for c in part.channels {
            match index.get(&c.id) {
                Some(&i) => {
                    for n in c.names {
                        if !out.channels[i].names.contains(&n) {
                            out.channels[i].names.push(n);
                        }
                    }
                }
                None => {
                    index.insert(c.id.clone(), out.channels.len());
                    out.channels.push(c);
                }
            }
        }
        for p in part.programmes {
            if seen.insert((p.channel.clone(), p.start)) {
                out.programmes.push(p);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests;
