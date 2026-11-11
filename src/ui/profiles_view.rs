use super::App;
use crate::model::ProfileId;

#[derive(Default)]
pub struct ProfilesDialog {
    new_name: String,
    renaming: Option<(ProfileId, String)>,
    confirm_delete: Option<ProfileId>,
}

enum Op {
    Create(String),
    Rename(ProfileId, String),
    Delete(ProfileId),
}

impl App {
    pub(super) fn open_profiles(&mut self) {
        self.profiles_dialog.get_or_insert_with(Default::default);
    }

    pub(super) fn profiles_window(&mut self, ctx: &egui::Context) {
        let Some(mut d) = self.profiles_dialog.take() else {
            return;
        };
        let mut open = true;
        let mut op = None;
        let can_delete = self.profiles.len() > 1;
        egui::Window::new("Profiles")
            .open(&mut open)
            .show(ctx, |ui| {
                for p in &self.profiles {
                    ui.horizontal(|ui| match &mut d.renaming {
                        Some((id, text)) if *id == p.id => {
                            ui.text_edit_singleline(text);
                            if ui.button("Save").clicked() {
                                op = Some(Op::Rename(p.id, text.clone()));
                            }
                            if ui.button("Cancel").clicked() {
                                d.renaming = None;
                            }
                        }
                        _ => {
                            ui.label(&p.name);
                            if ui.button("Rename").clicked() {
                                d.renaming = Some((p.id, p.name.clone()));
                            }
                            if d.confirm_delete == Some(p.id) {
                                if ui.button("Really delete").clicked() {
                                    op = Some(Op::Delete(p.id));
                                }
                                if ui.button("Cancel").clicked() {
                                    d.confirm_delete = None;
                                }
                            } else if can_delete && ui.button("Delete").clicked() {
                                d.confirm_delete = Some(p.id);
                            }
                        }
                    });
                }
                ui.separator();
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut d.new_name);
                    if ui
                        .add_enabled(!d.new_name.trim().is_empty(), egui::Button::new("Create"))
                        .clicked()
                    {
                        op = Some(Op::Create(d.new_name.trim().to_string()));
                    }
                });
            });
        if let Some(op) = op {
            self.run_profile_op(op, &mut d);
        }
        if open {
            self.profiles_dialog = Some(d);
        }
    }

    fn run_profile_op(&mut self, op: Op, d: &mut ProfilesDialog) {
        let res = match op {
            Op::Create(name) => {
                d.new_name.clear();
                self.db.create_profile(&name).map(|_| ())
            }
            Op::Rename(id, name) => {
                d.renaming = None;
                self.db.rename_profile(id, name.trim())
            }
            Op::Delete(id) => {
                d.confirm_delete = None;
                if self.state.profile == Some(id) {
                    self.state.profile = None;
                    self.series = None;
                }
                self.db.delete_profile(id)
            }
        };
        self.report(res);
        self.reload_all();
    }
}
