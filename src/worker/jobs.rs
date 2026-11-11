//! Network jobs that report back to the UI: account check, connection test, media info.
use super::{Command, Event, Worker};
use crate::model::{MediaKind, Source, SourceId};
use crate::sources;
use anyhow::Result;
use std::thread;

fn text(e: anyhow::Error) -> String {
    format!("{e:#}")
}

impl Worker {
    pub(super) fn check_account(&mut self, id: SourceId) -> Result<()> {
        let source = self.db.source(id)?;
        let (tx, repaint) = (self.cmd_tx.clone(), self.repaint.clone());
        thread::spawn(move || {
            let res = sources::check_account(&source).map_err(text);
            let _ = tx.send(Command::AccountChecked(id, res));
            repaint();
        });
        Ok(())
    }

    pub(super) fn apply_account(
        &mut self,
        id: SourceId,
        res: Result<String, String>,
    ) -> Result<()> {
        let outcome = match res {
            Ok(json) => self.db.set_account_json(id, &json).map_err(text),
            Err(e) => Err(e),
        };
        self.emit(Event::AccountChecked(id, outcome));
        Ok(())
    }

    pub(super) fn test_source(&mut self, source: Source) {
        let (tx, repaint) = (self.ev_tx.clone(), self.repaint.clone());
        thread::spawn(move || {
            let _ = tx.send(Event::TestResult(
                sources::probe::test_connection(&source).map_err(text),
            ));
            repaint();
        });
    }

    pub(super) fn load_info(
        &mut self,
        source: SourceId,
        media: MediaKind,
        key: String,
    ) -> Result<()> {
        let src = self.db.source(source)?;
        let (tx, repaint) = (self.ev_tx.clone(), self.repaint.clone());
        thread::spawn(move || {
            let result = match media {
                MediaKind::Series => sources::fetch_series(&src, &key).map(|(info, _)| info),
                _ => sources::fetch_vod_info(&src, &key),
            };
            let result = result.map_err(text);
            let _ = tx.send(Event::InfoLoaded {
                source,
                media,
                key,
                result,
            });
            repaint();
        });
        Ok(())
    }
}
