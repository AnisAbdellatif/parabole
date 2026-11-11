//! The Sources window: list with account panels, and the add/edit form.
mod account;
mod form;
mod list;

use super::App;
use crate::model::SourceId;
use crate::worker::Command;
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Default)]
pub struct SourcesDialog {
    editing: Option<SourceId>,
    paste: String,
    detected: String,
    name: String,
    url: String,
    user: String,
    pass: String,
    ua: String,
    mirrors: String,
    confirm_delete: Option<SourceId>,
    testing: bool,
    test_result: Option<Result<String, String>>,
    checking: HashSet<SourceId>,
    auto_hours: u32,
}

impl SourcesDialog {
    fn new(auto_hours: u32) -> Self {
        Self {
            auto_hours,
            ..Default::default()
        }
    }

    /// Clears the form but keeps state that belongs to the whole window.
    fn reset_form(&mut self) {
        *self = Self {
            checking: std::mem::take(&mut self.checking),
            auto_hours: self.auto_hours,
            ..Default::default()
        };
    }

    pub fn account_checked(&mut self, id: SourceId) {
        self.checking.remove(&id);
    }

    pub fn test_finished(&mut self, result: Result<String, String>) {
        self.testing = false;
        self.test_result = Some(result);
    }
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64)
}

fn ago(ts: i64) -> String {
    match now() - ts {
        s if s < 90 => "just now".into(),
        s if s < 5400 => format!("{} min ago", s / 60),
        s if s < 129_600 => format!("{} h ago", s / 3600),
        s => format!("{} d ago", s / 86_400),
    }
}

impl App {
    pub(super) fn open_sources(&mut self) {
        let hours = crate::worker::auto_refresh::hours(&self.db);
        self.sources_dialog
            .get_or_insert_with(|| SourcesDialog::new(hours));
    }

    pub(super) fn sources_window(&mut self, ctx: &egui::Context) {
        let Some(mut d) = self.sources_dialog.take() else {
            return;
        };
        let mut open = true;
        let mut cmds: Vec<Command> = Vec::new();
        let mut error = None;
        egui::Window::new("Sources")
            .open(&mut open)
            .default_width(560.0)
            .show(ctx, |ui| {
                self.source_list(ui, &mut d, &mut cmds);
                ui.separator();
                ui.heading(if d.editing.is_some() {
                    "Edit source"
                } else {
                    "Add source"
                });
                error = form::source_form(ui, &mut d, &mut cmds, self.profile());
            });
        for c in cmds {
            self.worker.send(c);
        }
        if let Some(e) = error {
            self.toast(e);
        }
        if open {
            self.sources_dialog = Some(d);
        }
    }
}
