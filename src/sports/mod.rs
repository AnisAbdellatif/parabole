//! Sports fixtures and TV channels from free, unofficial public endpoints. No DB, no UI.
//! Each provider lives in its own directory; nothing provider-specific leaks out of it.
pub mod catalog;
pub mod lang;
pub mod livescore;
pub mod merge;
pub mod registry;
pub mod s365;
pub mod sportsdb;
#[cfg(test)]
mod tests_live;

use anyhow::{Context, Result};
use reqwest::blocking::Client;
use std::io::Read;

/// Browser-like agent: both providers serve their public site's JSON to browsers.
pub const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0 Safari/537.36";
const MAX_BODY: u64 = 8 * 1024 * 1024;

pub fn client() -> Result<Client> {
    crate::sources::http::client(Some(USER_AGENT))
}

/// "Channel to be confirmed" entries some providers list before the real channel is known.
pub(crate) fn is_placeholder(channel: &str) -> bool {
    const WORDS: &[&str] = &[
        "da confermare",
        "to be confirmed",
        "tbc",
        "tba",
        "por confirmar",
        "a confirmar",
        "à confirmer",
        "a confirmer",
        "noch offen",
    ];
    let c = channel.trim().to_lowercase();
    WORDS
        .iter()
        .any(|w| c == *w || c.contains(&format!("{w} ")) || c.ends_with(&format!(" {w}")))
}

pub(crate) fn get_text(client: &Client, url: &str) -> Result<String> {
    let resp = crate::sources::http::get(client, url, Some(USER_AGENT))?;
    let mut body = String::new();
    crate::sources::http::SafeRead(resp)
        .take(MAX_BODY)
        .read_to_string(&mut body)
        .context("read response")?;
    Ok(body)
}

#[cfg(test)]
mod tests {
    #[test]
    fn placeholders() {
        assert!(super::is_placeholder("Canale da confermare"));
        assert!(super::is_placeholder("TBA"));
        assert!(!super::is_placeholder("beIN Sport 1 HD"));
        assert!(!super::is_placeholder("Tbilisi TV"));
    }
}
