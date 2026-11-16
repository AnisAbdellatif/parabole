//! TV guide refresh: one guide job at a time, fetched off-thread, written here.
use super::auto_refresh::now;
use super::{Command, Event, Worker};
use crate::epg::{self, Window};
use crate::model::{Programme, SourceId};
use crate::sources;
use anyhow::Result;
use std::collections::HashSet;
use std::thread;

const KEEP_PAST_SECS: i64 = 6 * 3600;

struct Job {
    urls: Vec<String>,
    user_agent: Option<String>,
    channels: HashSet<String>,
}

impl Worker {
    pub(super) fn queue_epg(&mut self, id: SourceId) {
        if self.epg_busy == Some(id) || self.epg_queue.contains(&id) {
            return;
        }
        self.epg_queue.push_back(id);
        self.next_epg();
    }

    fn next_epg(&mut self) {
        while self.epg_busy.is_none() {
            let Some(id) = self.epg_queue.pop_front() else {
                return;
            };
            match self.epg_job(id) {
                // a deleted source errors here: skip it
                Err(_) => {}
                Ok(None) => {
                    let done = self.db.replace_epg(id, &[], now());
                    self.emit(Event::EpgUpdated(id, done.map_err(|e| format!("{e:#}"))));
                }
                Ok(Some(job)) => {
                    self.epg_busy = Some(id);
                    let (tx, repaint) = (self.cmd_tx.clone(), self.repaint.clone());
                    thread::spawn(move || {
                        let res = epg::fetch_all(
                            &job.urls,
                            job.user_agent.as_deref(),
                            Window::around(now()),
                            &mut |_| job.channels.clone(),
                        )
                        .map(|g| g.programmes)
                        .map_err(|e| format!("{e:#}"));
                        let _ = tx.send(Command::EpgFetched(id, res));
                        repaint();
                    });
                }
            }
        }
    }

    /// `None` when there is nothing to fetch (no guide URL or no live channel with a guide id).
    fn epg_job(&self, id: SourceId) -> Result<Option<Job>> {
        let source = self.db.source(id)?;
        let epg_url = self.db.source_epg_url(id)?;
        let urls = sources::guide::guide_urls(&source, epg_url.as_deref()).unwrap_or_default();
        let channels = self.db.epg_channels(id)?;
        if urls.is_empty() || channels.is_empty() {
            return Ok(None);
        }
        Ok(Some(Job {
            urls,
            user_agent: source.user_agent,
            channels,
        }))
    }

    pub(super) fn apply_epg(&mut self, id: SourceId, res: Result<Vec<Programme>, String>) {
        self.epg_busy = None;
        let now = now();
        let outcome = res.and_then(|progs| {
            self.db
                .replace_epg(id, &progs, now)
                .and_then(|()| self.db.epg_prune(now - KEEP_PAST_SECS))
                .map_err(|e| format!("{e:#}"))
        });
        self.emit(Event::EpgUpdated(id, outcome));
        self.next_epg();
    }
}
