//! TV guide refresh. TODO(epg2): worker rewrite; for now a no-op that reports success.
use super::{Event, Worker};
use crate::model::{Programme, SourceId};

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
            // TODO(epg2): worker rewrite
            self.emit(Event::EpgUpdated(id, Ok(())));
        }
    }

    pub(super) fn apply_epg(&mut self, _id: SourceId, _res: Result<Vec<Programme>, String>) {
        self.epg_busy = None;
        self.next_epg();
    }
}
