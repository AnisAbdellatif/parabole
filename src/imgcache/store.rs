//! On-disk layout of the image cache: URL hashing, atomic writes, size-capped pruning.
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;

pub const MAX_IMAGE_BYTES: u64 = 5 * 1024 * 1024;
pub const MAX_CACHE_BYTES: u64 = 500 * 1024 * 1024;

/// Stable 64-bit FNV-1a of the URL as 16 hex chars (a collision only shows a wrong logo).
pub fn file_name(url: &str) -> String {
    let hash = url.bytes().fold(0xcbf2_9ce4_8422_2325_u64, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
    });
    format!("{hash:016x}")
}

/// Cached bytes for `path`; an oversized file is deleted. A hit refreshes the mtime for pruning.
pub fn read_cached(path: &Path) -> Option<Vec<u8>> {
    let len = fs::metadata(path).ok()?.len();
    if len > MAX_IMAGE_BYTES {
        let _ = fs::remove_file(path);
        return None;
    }
    let bytes = fs::read(path).ok()?;
    if let Ok(file) = fs::File::options().write(true).open(path) {
        let _ = file.set_modified(SystemTime::now());
    }
    Some(bytes)
}

pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let tmp = path.with_extension(format!("{}.{n}.tmp", std::process::id()));
    fs::write(&tmp, bytes).context("write cache file")?;
    fs::rename(&tmp, path)
        .context("move cache file into place")
        .inspect_err(|_| {
            let _ = fs::remove_file(&tmp);
        })
}

struct Entry {
    modified: SystemTime,
    len: u64,
    path: PathBuf,
}

/// Oldest-first files to delete so the remaining total fits in `max`.
fn plan_prune(mut entries: Vec<Entry>, max: u64) -> Vec<PathBuf> {
    entries.sort_by_key(|e| e.modified);
    let mut total: u64 = entries.iter().map(|e| e.len).sum();
    let mut doomed = Vec::new();
    for e in entries {
        if total <= max {
            break;
        }
        total -= e.len;
        doomed.push(e.path);
    }
    doomed
}

/// Deletes the oldest files until the directory is at most `max` bytes.
pub fn prune(dir: &Path, max: u64) -> Result<usize> {
    let mut entries = Vec::new();
    for item in fs::read_dir(dir).context("read image cache dir")? {
        let item = item.context("read image cache entry")?;
        let Ok(meta) = item.metadata() else { continue };
        if meta.is_file() {
            entries.push(Entry {
                modified: meta.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                len: meta.len(),
                path: item.path(),
            });
        }
    }
    let doomed = plan_prune(entries, max);
    Ok(doomed.iter().filter(|p| fs::remove_file(p).is_ok()).count())
}

#[cfg(test)]
mod tests;
