//! Row/menu actions that touch custom groups and alternate sources.
use super::App;
use crate::model::StreamRow;
use crate::worker::Command;

impl App {
    pub(super) fn add_to_group(&mut self, group: i64, row: &StreamRow) {
        let res = self.db.add_to_group(group, row.source_id, &row.key);
        self.report(res);
        self.reload_groups();
    }

    pub(super) fn remove_from_group(&mut self, row: &StreamRow) {
        if let Some(super::state::Sel::Group(g)) = self.state.sel {
            let res = self.db.remove_from_group(g, row.source_id, &row.key);
            self.report(res);
            self.reload_groups();
            self.reload_streams();
        }
    }

    pub(super) fn load_alternates(&mut self, row: &StreamRow) {
        match self.db.alternates(self.profile(), row.id) {
            Ok(alts) => self.alts = Some((row.id, alts)),
            Err(e) => self.toast(format!("{e:#}")),
        }
    }

    pub(super) fn play_alternate(&mut self, stream_id: i64) {
        self.worker.send(Command::Play {
            profile: self.profile(),
            stream_id,
            kind: None,
        });
    }

    pub(super) fn delete_group(&mut self, id: i64) {
        let res = self.db.delete_group(id);
        self.report(res);
        self.reload_categories();
    }

    pub(super) fn save_group(&mut self, id: Option<i64>, name: &str, add: Option<(i64, String)>) {
        let profile = self.profile();
        let res = match id {
            Some(id) => self.db.rename_group(id, name),
            None => self
                .db
                .create_group(profile, self.state.tab, name)
                .and_then(|g| match add {
                    Some((source, key)) => self.db.add_to_group(g, source, &key),
                    None => Ok(()),
                }),
        };
        self.report(res);
        self.reload_groups();
    }

    pub(super) fn reload_groups(&mut self) {
        match self.db.groups(self.profile(), self.state.tab) {
            Ok(g) => self.groups = g,
            Err(e) => self.toast(format!("{e:#}")),
        }
    }
}
