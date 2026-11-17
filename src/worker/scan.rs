//! Background quality scanner: probes the selected live channels one at a time with ffprobe.

use super::{Event, Repaint, play::stream_request};
use crate::db::Db;
use crate::model::ProfileId;
use crate::player::probe::{ProbeError, find_ffprobe, probe};
use anyhow::{Context, Result};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const BATCH: i64 = 20;

/// Shared between the UI, the worker and the scanner thread.
#[derive(Default)]
pub struct ScanControl {
    pub profile: Mutex<Option<ProfileId>>,
    pub paused: AtomicBool,
    pub embedded_playing: AtomicBool,
    pub external_playing: AtomicBool,
}

impl ScanControl {
    fn blocked(&self) -> bool {
        self.paused.load(Ordering::Relaxed)
            || self.embedded_playing.load(Ordering::Relaxed)
            || self.external_playing.load(Ordering::Relaxed)
    }

    fn profile(&self) -> Option<ProfileId> {
        *self.profile.lock().unwrap_or_else(|e| e.into_inner())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ScanStatus {
    Unavailable,
    Running { done: i64, total: i64 },
    Idle { done: i64, total: i64 },
}

pub(super) fn spawn(
    db_path: &Path,
    tx: Sender<Event>,
    repaint: Repaint,
) -> Result<Arc<ScanControl>> {
    let db = Db::open(db_path).context("open scanner database")?;
    let ctl = Arc::new(ScanControl::default());
    let mut scanner = Scanner {
        db,
        ctl: ctl.clone(),
        tx,
        repaint,
        last: None,
    };
    thread::Builder::new()
        .name("quality-scan".into())
        .spawn(move || scanner.run())
        .context("spawn scanner thread")?;
    Ok(ctl)
}

struct Scanner {
    db: Db,
    ctl: Arc<ScanControl>,
    tx: Sender<Event>,
    repaint: Repaint,
    last: Option<ScanStatus>,
}

impl Scanner {
    fn emit(&mut self, s: ScanStatus) {
        if self.last.as_ref() == Some(&s) {
            return;
        }
        self.last = Some(s.clone());
        if self.tx.send(Event::ScanStatus(s)).is_ok() {
            (self.repaint)();
        }
    }

    fn run(&mut self) {
        loop {
            let Some(bin) = find_ffprobe() else {
                self.emit(ScanStatus::Unavailable);
                thread::sleep(Duration::from_secs(30));
                continue;
            };
            let wait = match self.ctl.profile() {
                Some(p) if !self.ctl.blocked() => self.step(&bin, p),
                _ => Duration::from_secs(1),
            };
            thread::sleep(wait);
        }
    }

    /// One batch; returns how long to sleep afterwards.
    fn step(&mut self, bin: &Path, profile: ProfileId) -> Duration {
        let ids = match self.db.scan_queue(profile, BATCH) {
            Ok(ids) => ids,
            Err(_) => return Duration::from_secs(30),
        };
        if ids.is_empty() {
            if let Ok((done, total)) = self.db.scan_counts(profile) {
                self.emit(ScanStatus::Idle { done, total });
            }
            return Duration::from_secs(5);
        }
        for id in ids {
            if self.ctl.blocked() || self.ctl.profile() != Some(profile) {
                break;
            }
            match self.probe_one(bin, id) {
                Probed::Next => {}
                Probed::Cancelled => break,
                Probed::DbError(e) => {
                    eprintln!("quality scan: database write failed: {e:#}");
                    return Duration::from_secs(30);
                }
            }
            if let Ok((done, total)) = self.db.scan_counts(profile) {
                self.emit(ScanStatus::Running { done, total });
            }
            thread::sleep(Duration::from_secs(1));
        }
        Duration::ZERO
    }

    fn probe_one(&self, bin: &Path, id: i64) -> Probed {
        let Ok((info, req)) = stream_request(&self.db, id) else {
            // Unresolvable URL: mark it so the queue moves on.
            if let Ok(info) = self.db.play_info(id)
                && let Err(e) = self.db.record_failure(info.source_id, &info.key)
            {
                return Probed::DbError(e);
            }
            return Probed::Next;
        };
        let ctl = self.ctl.clone();
        let saved = match probe(bin, &req, &move || ctl.blocked()) {
            Ok(q) => self.db.save_quality(info.source_id, &info.key, &q),
            Err(ProbeError::Cancelled) => return Probed::Cancelled,
            Err(ProbeError::Failed(_)) => self.db.record_failure(info.source_id, &info.key),
        };
        saved.map_or_else(Probed::DbError, |_| Probed::Next)
    }
}

enum Probed {
    Next,
    /// Playback started mid-probe: abandon the batch.
    Cancelled,
    /// Never carries a URL (they hold credentials), so it is safe to print.
    DbError(anyhow::Error),
}

#[cfg(all(test, unix))]
mod tests;
