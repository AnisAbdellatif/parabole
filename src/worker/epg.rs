//! TV guide refresh: one guide job at a time, then relinking streams to guide channels.
use super::auto_refresh::now;
use super::{Command, Event, Worker};
use crate::db::Db;
use crate::epg::{self, Window, matching};
use crate::model::GuideId;
use crate::sources::guide::guide_urls;
use anyhow::Result;
use std::thread;

impl Worker {
    pub(super) fn queue_epg(&mut self, id: GuideId) {
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
            match self.start_epg(id) {
                Ok(true) => self.epg_busy = Some(id),
                Ok(false) => {}
                Err(e) => self.emit(Event::Error(format!("{e:#}"))),
            }
        }
    }

    /// Ok(true) when a fetch thread was spawned; Ok(false) when the job finished or was skipped.
    fn start_epg(&mut self, id: GuideId) -> Result<bool> {
        let Ok(guide) = self.db.epg_guide(id) else {
            return Ok(false);
        };
        if !guide.enabled {
            return Ok(false);
        }
        let (urls, ua) = match (guide.source_id, guide.url) {
            (Some(sid), _) => match self.db.source(sid) {
                Ok(src) => {
                    let epg_url = self.db.source_epg_url(sid)?;
                    let urls = guide_urls(&src, epg_url.as_deref()).unwrap_or_default();
                    (urls, src.user_agent)
                }
                Err(_) => (Vec::new(), None),
            },
            (None, Some(url)) => (vec![url], None),
            (None, None) => (Vec::new(), None),
        };
        let now = now();
        if urls.is_empty() {
            self.db.replace_guide(id, &[], &[], now)?;
            relink(&mut self.db)?;
            self.emit(Event::EpgUpdated(id, Ok(())));
            return Ok(false);
        }
        let streams = self.db.epg_stream_keys()?;
        let (tx, repaint) = (self.cmd_tx.clone(), self.repaint.clone());
        thread::spawn(move || {
            let res = epg::fetch_all(&urls, ua.as_deref(), Window::around(now), &mut |chans| {
                matching::wanted(&streams, chans)
            })
            .map_err(|e| format!("{e:#}"));
            let _ = tx.send(Command::EpgFetched(id, res));
            repaint();
        });
        Ok(true)
    }

    pub(super) fn apply_epg(&mut self, id: GuideId, res: Result<epg::ParsedGuide, String>) {
        self.epg_busy = None;
        let now = now();
        let outcome = match res {
            Ok(g) => self
                .db
                .replace_guide(id, &g.channels, &g.programmes, now)
                .and_then(|_| self.db.epg_prune(now - 6 * 3600))
                .and_then(|_| relink(&mut self.db).map(|_| ()))
                .map_err(|e| format!("{e:#}")),
            Err(e) => {
                let _ = self.db.set_epg_guide_error(id, &e, now);
                Err(e)
            }
        };
        self.emit(Event::EpgUpdated(id, outcome));
        self.next_epg();
    }

    pub(super) fn relink_epg(&mut self) {
        match relink(&mut self.db) {
            Ok(_) => self.emit(Event::EpgRelinked),
            Err(e) => self.emit(Event::Error(format!("{e:#}"))),
        }
    }
}

/// Recomputes automatic stream↔guide links; returns how many there are.
pub fn relink(db: &mut Db) -> Result<usize> {
    let streams = db.epg_stream_keys()?;
    let candidates: Vec<_> = db
        .epg_candidates()?
        .into_iter()
        .map(|(guide_id, channel, names)| matching::Candidate {
            guide_id,
            channel,
            names,
        })
        .collect();
    let links = matching::link(&streams, &candidates);
    db.replace_auto_links(&links)?;
    Ok(links.len())
}
