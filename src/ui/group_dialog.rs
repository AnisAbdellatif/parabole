//! Small modal to name a new group or rename one.
use super::App;
use super::theme::{primary_button, secondary_button};
use crate::model::SourceId;

pub struct GroupDialog {
    id: Option<i64>,
    name: String,
    /// Item to put into the group right after creating it.
    add: Option<(SourceId, String)>,
    focused: bool,
}

impl App {
    pub(super) fn open_group_dialog(
        &mut self,
        id: Option<i64>,
        name: String,
        add: Option<(SourceId, String)>,
    ) {
        self.group_dialog = Some(GroupDialog {
            id,
            name,
            add,
            focused: false,
        });
    }

    pub(super) fn group_window(&mut self, ctx: &egui::Context) {
        let Some(mut d) = self.group_dialog.take() else {
            return;
        };
        let (mut open, mut done, mut cancel) = (true, false, false);
        let title = if d.id.is_some() {
            "Rename group"
        } else {
            "New group"
        };
        egui::Window::new(title)
            .collapsible(false)
            .resizable(false)
            .open(&mut open)
            .show(ctx, |ui| {
                let edit = ui.text_edit_singleline(&mut d.name);
                if !d.focused {
                    edit.request_focus();
                    d.focused = true;
                }
                done = edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                ui.horizontal(|ui| {
                    done |= primary_button(ui, "Save").clicked();
                    if secondary_button(ui, "Cancel").clicked() {
                        cancel = true;
                    }
                });
            });
        let name = d.name.trim().to_string();
        if done && !name.is_empty() {
            self.save_group(d.id, &name, d.add.take());
        } else if open && !cancel {
            self.group_dialog = Some(d);
        }
    }
}
