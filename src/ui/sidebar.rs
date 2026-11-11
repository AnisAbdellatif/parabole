use super::App;
use super::state::Sel;
use crate::model::CategoryRow;

impl App {
    pub(super) fn sidebar(&mut self, ui: &mut egui::Ui) {
        let panel = egui::Panel::left("sidebar")
            .resizable(true)
            .default_size(self.state.sidebar_width)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Categories");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.selectable_label(self.edit.on, "Edit").clicked() {
                            self.edit.on = !self.edit.on;
                            self.reload_categories();
                        }
                    });
                });
                if self.edit.on {
                    self.edit_ui(ui);
                } else {
                    self.normal_ui(ui);
                }
            });
        self.state.sidebar_width = panel.response.rect.width();
    }

    pub(super) fn cat_label(&self, c: &CategoryRow) -> String {
        if self.show_source_prefix() {
            format!("{}: {}", self.source_name(c.source_id), c.name)
        } else {
            c.name.clone()
        }
    }

    pub(super) fn select(&mut self, sel: Sel) {
        self.state.sel = Some(sel);
        self.series = None;
        self.selected = 0;
        self.search.text.clear();
        self.search.applied.clear();
        self.reload_streams();
    }

    fn normal_ui(&mut self, ui: &mut egui::Ui) {
        let mut picked = None;
        let mut pin = None;
        for (label, sel) in [("★ Favorites", Sel::Favorites), ("⟲ Recent", Sel::Recent)] {
            if ui
                .selectable_label(self.state.sel.as_ref() == Some(&sel), label)
                .clicked()
            {
                picked = Some(sel);
            }
        }
        ui.separator();
        let row_h = ui.spacing().interact_size.y;
        egui::ScrollArea::vertical().auto_shrink(false).show_rows(
            ui,
            row_h,
            self.categories.len(),
            |ui, range| {
                for c in &self.categories[range] {
                    ui.horizontal(|ui| {
                        let star = if c.pinned { "★" } else { "☆" };
                        if ui.small_button(star).on_hover_text("Pin to top").clicked() {
                            pin = Some((c.source_id, c.key.clone(), !c.pinned));
                        }
                        let sel = Sel::Category(c.source_id, c.key.clone());
                        let text = format!("{}  ({})", self.cat_label(c), c.count);
                        if ui
                            .selectable_label(self.state.sel.as_ref() == Some(&sel), text)
                            .clicked()
                        {
                            picked = Some(sel);
                        }
                    });
                }
            },
        );
        if let Some((source, key, pinned)) = pin {
            let res =
                self.db
                    .set_category_pinned(self.profile(), source, self.state.tab, &key, pinned);
            self.report(res);
            self.reload_categories();
        }
        if let Some(sel) = picked {
            self.select(sel);
        }
    }

    pub(super) fn report(&mut self, res: anyhow::Result<()>) {
        if let Err(e) = res {
            self.toast(format!("{e:#}"));
        }
    }
}
