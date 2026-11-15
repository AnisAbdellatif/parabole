//! Disk cache for http(s) images (logos, posters): fetch, store, prune.
mod fetch;
mod store;

use anyhow::{Context as _, Result};
use directories::ProjectDirs;
use std::path::{Path, PathBuf};

/// Trims `dir` to the size cap on a background thread.
pub fn spawn_prune(dir: PathBuf) {
    std::thread::spawn(move || {
        let _ = store::prune(&dir, store::MAX_CACHE_BYTES);
    });
}

pub use fetch::client as http_client;

pub fn default_dir() -> Option<PathBuf> {
    ProjectDirs::from("dev", "parabole", "Parabole").map(|d| d.cache_dir().join("images"))
}

/// Cached bytes for `url`, downloading and storing them on a miss. Blocking; call off the UI thread.
pub fn get_or_fetch(dir: &Path, client: &reqwest::blocking::Client, url: &str) -> Result<Vec<u8>> {
    let path = dir.join(store::file_name(url));
    if let Some(bytes) = store::read_cached(&path) {
        return Ok(bytes);
    }
    let bytes = fetch::download(client, url)?;
    std::fs::create_dir_all(dir).context("create image cache dir")?;
    store::write_atomic(&path, &bytes)?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_or_fetch_hits_cache_without_network() {
        let dir = std::env::temp_dir().join(format!("imgcache-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let url = "http://127.0.0.1:1/never.png";
        store::write_atomic(&dir.join(store::file_name(url)), b"png").unwrap();
        let client = http_client().unwrap();
        assert_eq!(get_or_fetch(&dir, &client, url).unwrap(), b"png");
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
