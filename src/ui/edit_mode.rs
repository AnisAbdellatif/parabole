use super::App;
use crate::model::CategoryRow;
use egui_dnd::{DragDropItem, dnd};

struct Item<'a>(&'a CategoryRow);

impl DragDropItem for Item<'_> {
    fn id(&self) -> egui::Id {
        egui::Id::unique(("cat", self.0.source_id, &self.0.key))
    }
}

enum Action {
    Hidden(usize, bool),
    Pinned(usize, bool),
}

impl App {
    pub(super) fn edit_ui(&mut self, ui: &mut egui::Ui) {
        self.edit_toolbar(ui);
        let single = self.single_source();
        if single.is_none() {
            ui.weak("Select one source to reorder categories.");
        }
        let mut actions = Vec::new();
        let mut finished = false;
        let tab = self.state.tab;
        let profile = self.profile();
        egui::ScrollArea::vertical()
            .auto_shrink(false)
            .show(ui, |ui| {
                let response = dnd(ui, egui::Id::unique("cat_dnd")).show(
                    self.categories.iter().map(Item),
                    |ui, item, handle, state| {
                        let c = item.0;
                        ui.horizontal(|ui| {
                            if single.is_some() {
                                handle.ui(ui, |ui| {
                                    ui.label("☰");
                                });
                            }
                            let eye = if c.hidden { "🚫" } else { "👁" };
                            if ui.small_button(eye).on_hover_text("Show / hide").clicked() {
                                actions.push(Action::Hidden(state.index, !c.hidden));
                            }
                            let star = if c.pinned { "★" } else { "☆" };
                            if ui.small_button(star).on_hover_text("Pin to top").clicked() {
                                actions.push(Action::Pinned(state.index, !c.pinned));
                            }
                            let text =
                                egui::RichText::new(format!("{} ({})", self.cat_label(c), c.count));
                            ui.label(if c.hidden { text.weak() } else { text });
                        });
                    },
                );
                finished = response.is_drag_finished();
                if single.is_some() {
                    response.update_vec(&mut self.categories);
                }
            });
        for a in actions {
            let (idx, hidden, pinned) = match a {
                Action::Hidden(i, h) => (i, Some(h), None),
                Action::Pinned(i, p) => (i, None, Some(p)),
            };
            let Some(c) = self.categories.get(idx) else {
                continue;
            };
            let (s, k) = (c.source_id, c.key.clone());
            let res = match (hidden, pinned) {
                (Some(h), _) => self.db.set_category_hidden(profile, s, tab, &k, h),
                (_, Some(p)) => self.db.set_category_pinned(profile, s, tab, &k, p),
                _ => Ok(()),
            };
            self.report(res);
            self.reload_categories();
        }
        if let (true, Some(source)) = (finished, single) {
            let keys: Vec<String> = self.categories.iter().map(|c| c.key.clone()).collect();
            let res = self.db.reorder_categories(profile, source, tab, &keys);
            self.report(res);
            self.reload_categories();
        }
    }

    fn edit_toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            if ui.button("Show all").clicked() {
                self.bulk_hidden(|_| false);
            }
            if ui.button("Hide all").clicked() {
                self.bulk_hidden(|_| true);
            }
            let single = self.single_source();
            let reset = ui.add_enabled(single.is_some(), egui::Button::new("Reset order"));
            let sort = ui.add_enabled(single.is_some(), egui::Button::new("Sort A–Z"));
            if let (true, Some(s)) = (sort.clicked(), single) {
                let mut sorted: Vec<_> = self.categories.iter().collect();
                sorted.sort_by_cached_key(|c| c.name.to_lowercase());
                let keys: Vec<String> = sorted.iter().map(|c| c.key.clone()).collect();
                let res = self
                    .db
                    .reorder_categories(self.profile(), s, self.state.tab, &keys);
                self.report(res);
                self.reload_categories();
            }
            if let (true, Some(s)) = (reset.clicked(), single) {
                let res = self
                    .db
                    .reset_category_order(self.profile(), s, self.state.tab);
                self.report(res);
                self.reload_categories();
            }
        });
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.edit.filter)
                    .hint_text("Filter, e.g. FR |")
                    .desired_width(110.0),
            );
            let needle = self.edit.filter.to_lowercase();
            let has = !needle.is_empty();
            if ui
                .add_enabled(has, egui::Button::new("Hide matching"))
                .clicked()
            {
                self.bulk_hidden(|name| name.to_lowercase().contains(&needle));
            }
            if ui
                .add_enabled(has, egui::Button::new("Show only matching"))
                .clicked()
            {
                self.bulk_hidden(|name| !name.to_lowercase().contains(&needle));
            }
        });
        ui.separator();
    }

    fn bulk_hidden(&mut self, hide: impl Fn(&str) -> bool) {
        let changes: Vec<_> = self
            .categories
            .iter()
            .filter_map(|c| {
                let want = hide(&c.name);
                (want != c.hidden).then(|| (c.source_id, c.key.clone(), want))
            })
            .collect();
        let res = self
            .db
            .set_categories_hidden(self.profile(), self.state.tab, &changes);
        self.report(res);
        self.reload_categories();
    }
}
