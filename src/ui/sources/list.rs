use super::{SourcesDialog, account, ago, now};
use crate::model::SourceKind;
use crate::ui::App;
use crate::worker::{Command, auto_refresh};

impl App {
    pub(super) fn source_list(
        &self,
        ui: &mut egui::Ui,
        d: &mut SourcesDialog,
        cmds: &mut Vec<Command>,
    ) {
        if self.sources.is_empty() {
            ui.weak("No sources yet.");
        }
        let now = now();
        for s in &self.sources {
            ui.horizontal(|ui| {
                ui.strong(&s.name);
                ui.weak(s.kind.as_str());
                if self.refreshing.contains(&s.id) {
                    ui.spinner();
                } else if let Some(ts) = s.last_refresh {
                    ui.weak(format!("refreshed {}", ago(ts)));
                }
                if ui
                    .add_enabled(
                        !self.refreshing.contains(&s.id),
                        egui::Button::new("Refresh"),
                    )
                    .clicked()
                {
                    cmds.push(Command::Refresh(s.id));
                }
                if s.kind == SourceKind::Xtream {
                    let checking = d.checking.contains(&s.id);
                    if ui
                        .add_enabled(!checking, egui::Button::new("Check account"))
                        .clicked()
                    {
                        d.checking.insert(s.id);
                        cmds.push(Command::CheckAccount(s.id));
                    }
                    if checking {
                        ui.spinner();
                    }
                }
                if ui.button("Edit").clicked() {
                    d.load(s);
                }
                if d.confirm_delete == Some(s.id) {
                    if ui.button("Really delete").clicked() {
                        cmds.push(Command::DeleteSource(s.id));
                        d.confirm_delete = None;
                    }
                    if ui.button("Cancel").clicked() {
                        d.confirm_delete = None;
                    }
                } else if ui.button("Delete").clicked() {
                    d.confirm_delete = Some(s.id);
                }
            });
            if let Some(info) = self.accounts.get(&s.id) {
                account::panel(ui, info, now);
            }
            if let Some(e) = &s.last_error {
                ui.colored_label(egui::Color32::LIGHT_RED, e);
            }
        }
        if self.sources.len() > 1 && ui.button("Refresh all").clicked() {
            cmds.push(Command::RefreshAll(self.profile()));
        }
        ui.horizontal(|ui| {
            ui.label("Auto-refresh at startup after (hours, 0 = off):");
            let r = ui.add(egui::DragValue::new(&mut d.auto_hours).range(0..=720));
            if r.drag_stopped() || r.lost_focus() {
                let _ = auto_refresh::set_hours(&self.db, d.auto_hours);
            }
        });
    }
}
