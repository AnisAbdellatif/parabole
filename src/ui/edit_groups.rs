//! Edit-mode section of the sidebar: create, rename, delete and drag-reorder groups.
use super::App;
use super::theme::{self, Icon};
use crate::model::StreamGroup;
use egui::{Frame, Margin, RichText, Ui, vec2};
use egui_dnd::{DragDropItem, dnd};

struct Item<'a>(&'a StreamGroup);

impl DragDropItem for Item<'_> {
    fn id(&self) -> egui::Id {
        egui::Id::unique(("group", self.0.id))
    }
}

impl App {
    pub(super) fn edit_groups(&mut self, ui: &mut Ui) {
        let (mut new, mut finished) = (false, false);
        let (mut rename, mut delete) = (None, None);
        let mut confirm = self.edit.confirm_group;
        Frame::NONE
            .inner_margin(Margin {
                left: 12,
                right: 12,
                top: 0,
                bottom: 8,
            })
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("MY GROUPS")
                            .font(theme::semibold(11.0))
                            .color(theme::TEXT_DIM),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        new = ui.small_button("+ New group").clicked();
                    });
                });
                let response = dnd(ui, egui::Id::unique("group_dnd")).show(
                    self.groups.iter().map(Item),
                    |ui, item, handle, _| {
                        let g = item.0;
                        ui.horizontal(|ui| {
                            handle.ui_sized(ui, vec2(22.0, 28.0), |ui| {
                                ui.add(theme::icon(Icon::Grip, 14.0, theme::TEXT_FAINT));
                            });
                            ui.label(&g.name);
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if confirm == Some(g.id) {
                                        if ui.small_button("Delete?").clicked() {
                                            delete = Some(g.id);
                                        }
                                    } else if ui.small_button("Delete").clicked() {
                                        confirm = Some(g.id);
                                    }
                                    if ui.small_button("Rename").clicked() {
                                        rename = Some((g.id, g.name.clone()));
                                    }
                                },
                            );
                        });
                    },
                );
                finished = response.is_drag_finished();
                response.update_vec(&mut self.groups);
            });
        self.edit.confirm_group = confirm;
        if finished {
            let ids: Vec<i64> = self.groups.iter().map(|g| g.id).collect();
            let res = self.db.reorder_groups(&ids);
            self.report(res);
        }
        if let Some(id) = delete {
            self.edit.confirm_group = None;
            self.delete_group(id);
        }
        if let Some((id, name)) = rename {
            self.open_group_dialog(Some(id), name, None);
        }
        if new {
            self.open_group_dialog(None, String::new(), None);
        }
    }
}
