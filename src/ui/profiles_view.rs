use super::App;
use super::theme::{primary_button, secondary_button};
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
    Duplicate(ProfileId, String),
    Delete(ProfileId),
    CopyFavorites { from: ProfileId, to: ProfileId },
}

fn copy_name(profiles: &[crate::model::Profile], base: &str) -> String {
    let taken = |n: &str| profiles.iter().any(|p| p.name == n);
    let mut name = format!("{base} copy");
    let mut i = 2;
    while taken(&name) {
        name = format!("{base} copy {i}");
        i += 1;
    }
    name
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
                            if primary_button(ui, "Save").clicked() {
                                op = Some(Op::Rename(p.id, text.clone()));
                            }
                            if secondary_button(ui, "Cancel").clicked() {
                                d.renaming = None;
                            }
                        }
                        _ => {
                            ui.label(&p.name);
                            if ui.button("Rename").clicked() {
                                d.renaming = Some((p.id, p.name.clone()));
                            }
                            if ui.button("Duplicate").clicked() {
                                op = Some(Op::Duplicate(p.id, copy_name(&self.profiles, &p.name)));
                            }
                            if self.profiles.len() > 1 {
                                ui.menu_button("Copy favorites from…", |ui| {
                                    for other in self.profiles.iter().filter(|o| o.id != p.id) {
                                        if ui.button(&other.name).clicked() {
                                            op = Some(Op::CopyFavorites {
                                                from: other.id,
                                                to: p.id,
                                            });
                                            ui.close();
                                        }
                                    }
                                });
                            }
                            if d.confirm_delete == Some(p.id) {
                                if ui.button("Really delete").clicked() {
                                    op = Some(Op::Delete(p.id));
                                }
                                if secondary_button(ui, "Cancel").clicked() {
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
                        .add_enabled_ui(!d.new_name.trim().is_empty(), |ui| {
                            primary_button(ui, "Create")
                        })
                        .inner
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
            Op::CopyFavorites { from, to } => self.db.copy_favorites(from, to).map(|n| {
                self.toast(format!(
                    "Copied {n} favorite{}",
                    if n == 1 { "" } else { "s" }
                ));
            }),
            Op::Create(name) => {
                d.new_name.clear();
                self.db.create_profile(&name).map(|_| ())
            }
            Op::Rename(id, name) => {
                d.renaming = None;
                self.db.rename_profile(id, name.trim())
            }
            Op::Duplicate(id, name) => self.db.duplicate_profile(id, &name).map(|_| ()),
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
