//! Disk-backed egui `BytesLoader` for http(s) images (logos, posters).
//! Misses download on a small worker pool; failures are remembered until restart.
mod fetch;
mod store;

use anyhow::{Context as _, Result};
use directories::ProjectDirs;
use egui::Context;
use egui::load::{Bytes, BytesLoadResult, BytesLoader, BytesPoll, LoadError};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

const WORKERS: usize = 8;

enum State {
    Pending,
    Failed(String),
}

struct Shared {
    dir: PathBuf,
    /// Keyed by `store::file_name`, so URLs (and any credentials in them) are not retained.
    state: Mutex<HashMap<String, State>>,
}

impl Shared {
    fn state(&self) -> MutexGuard<'_, HashMap<String, State>> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

type Job = (String, Context);

struct DiskCacheLoader {
    shared: Arc<Shared>,
    jobs: Sender<Job>,
}

fn is_http(uri: &str) -> bool {
    uri.starts_with("http://") || uri.starts_with("https://")
}

impl BytesLoader for DiskCacheLoader {
    fn id(&self) -> &str {
        egui::load::generate_loader_id!(DiskCacheLoader)
    }

    fn load(&self, ctx: &Context, uri: &str) -> BytesLoadResult {
        if !is_http(uri) {
            return Err(LoadError::NotSupported);
        }
        let key = store::file_name(uri);
        let mut state = self.shared.state();
        match state.get(&key) {
            Some(State::Pending) => return Ok(BytesPoll::Pending { size: None }),
            Some(State::Failed(msg)) => return Err(LoadError::Loading(msg.clone())),
            None => {}
        }
        if let Some(bytes) = store::read_cached(&self.shared.dir.join(&key)) {
            return Ok(BytesPoll::Ready {
                size: None,
                bytes: Bytes::Shared(bytes.into()),
                mime: None,
            });
        }
        if self.jobs.send((uri.to_owned(), ctx.clone())).is_err() {
            let msg = "image workers stopped".to_owned();
            state.insert(key, State::Failed(msg.clone()));
            return Err(LoadError::Loading(msg));
        }
        state.insert(key, State::Pending);
        Ok(BytesPoll::Pending { size: None })
    }

    fn forget(&self, uri: &str) {
        let key = store::file_name(uri);
        let mut state = self.shared.state();
        if matches!(state.get(&key), Some(State::Failed(_))) {
            state.remove(&key);
        }
    }

    fn forget_all(&self) {
        self.shared
            .state()
            .retain(|_, s| matches!(s, State::Pending));
    }

    fn byte_size(&self) -> usize {
        0
    }
}

fn worker(jobs: Arc<Mutex<Receiver<Job>>>, client: reqwest::blocking::Client, shared: Arc<Shared>) {
    loop {
        let job = match jobs.lock() {
            Ok(rx) => rx.recv(),
            Err(_) => return,
        };
        let Ok((url, ctx)) = job else { return };
        let key = store::file_name(&url);
        let result = fetch::download(&client, &url)
            .and_then(|bytes| store::write_atomic(&shared.dir.join(&key), &bytes));
        {
            let mut state = shared.state();
            match result {
                Ok(()) => state.remove(&key),
                Err(e) => state.insert(key, State::Failed(format!("{e:#}"))),
            };
        }
        ctx.request_repaint();
    }
}

/// Registers the loader (tried before egui_extras' http loader) and prunes `dir` in the background.
pub fn install(ctx: &Context, dir: PathBuf) -> Result<()> {
    std::fs::create_dir_all(&dir).context("create image cache dir")?;
    let client = fetch::client()?;
    let prune_dir = dir.clone();
    std::thread::spawn(move || {
        let _ = store::prune(&prune_dir, store::MAX_CACHE_BYTES);
    });
    let shared = Arc::new(Shared {
        dir,
        state: Mutex::default(),
    });
    let (jobs, rx) = channel();
    let rx = Arc::new(Mutex::new(rx));
    for _ in 0..WORKERS {
        let (rx, client, shared) = (rx.clone(), client.clone(), shared.clone());
        std::thread::spawn(move || worker(rx, client, shared));
    }
    ctx.add_bytes_loader(Arc::new(DiskCacheLoader { shared, jobs }));
    Ok(())
}

pub use fetch::client as http_client;

/// Cache dir shared by every UI toolkit.
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
