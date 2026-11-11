//! Bottom toolbar of the sidebar in edit mode: bulk show/hide, order tools, name filter.
use super::App;
use super::kit;
use super::theme;
use egui::{Frame, Margin, Stroke, Ui, vec2};

impl App {
    pub(super) fn edit_toolbar(&mut self, ui: &mut Ui) {
        let panel = egui::Panel::bottom("edit_toolbar")
            .resizable(false)
            .show_separator_line(false)
            .frame(
                Frame::NONE
                    .fill(theme::TOOLBAR)
                    .inner_margin(Margin::same(12)),
            )
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = vec2(6.0, 10.0);
                let single = self.single_source();
                let half = vec2((ui.available_width() - 6.0) / 2.0, theme::CONTROL_H);
                ui.horizontal(|ui| {
                    if kit::pill(ui, "Show all", half, false, true).clicked() {
                        self.bulk_hidden(|_| false);
                    }
                    if kit::pill(ui, "Hide all", half, false, true).clicked() {
                        self.bulk_hidden(|_| true);
                    }
                });
                ui.horizontal(|ui| {
                    let reset = kit::pill(ui, "Reset order", half, false, single.is_some());
                    let sort = kit::pill(ui, "Sort A–Z", half, false, single.is_some());
                    if let (true, Some(s)) = (sort.clicked(), single) {
                        let mut sorted: Vec<_> = self.categories.iter().collect();
                        sorted.sort_by_cached_key(|c| c.name.to_lowercase());
                        let keys: Vec<String> = sorted.iter().map(|c| c.key.clone()).collect();
                        let res =
                            self.db
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
                let needle = self.edit.filter.to_lowercase();
                let has = !needle.is_empty();
                ui.horizontal(|ui| {
                    let hide_w = kit::pill_width(ui, "Hide matching");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.edit.filter)
                            .hint_text("Filter, e.g. FR |")
                            .margin(Margin::symmetric(10, 8))
                            .desired_width(ui.available_width() - hide_w - 6.0),
                    );
                    let size = vec2(hide_w, theme::CONTROL_H);
                    if kit::pill(ui, "Hide matching", size, true, has).clicked() {
                        self.bulk_hidden(|name| name.to_lowercase().contains(&needle));
                    }
                });
                let size = vec2(ui.available_width(), theme::CONTROL_H);
                if kit::pill(ui, "Show only matching", size, false, has).clicked() {
                    self.bulk_hidden(|name| !name.to_lowercase().contains(&needle));
                }
                self.hide_new_toggle(ui);
            });
        let top = panel.response.rect.top();
        let rect = panel.response.rect;
        ui.painter()
            .hline(rect.x_range(), top, Stroke::new(1.0, theme::DIVIDER));
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
