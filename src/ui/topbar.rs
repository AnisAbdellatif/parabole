use super::App;
use crate::model::MediaKind;
use egui::{Key, Modifiers};

fn tab_label(kind: MediaKind) -> &'static str {
    match kind {
        MediaKind::Live => "Live",
        MediaKind::Movie => "Movies",
        MediaKind::Series => "Series",
    }
}

impl App {
    pub(super) fn topbar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("topbar").show(ui, |ui| {
            ui.horizontal(|ui| {
                self.profile_picker(ui);
                self.source_picker(ui);
                ui.separator();
                for kind in MediaKind::ALL {
                    if ui
                        .selectable_label(self.state.tab == kind, tab_label(kind))
                        .clicked()
                    {
                        self.set_tab(kind);
                    }
                }
                ui.separator();
                self.search_box(ui);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Settings…").clicked() {
                        self.open_settings();
                    }
                    if ui.button("Sources…").clicked() {
                        self.open_sources();
                    }
                });
            });
        });
    }

    fn profile_picker(&mut self, ui: &mut egui::Ui) {
        let current = self
            .profiles
            .iter()
            .find(|p| Some(p.id) == self.state.profile)
            .map_or("?", |p| p.name.as_str())
            .to_string();
        let mut chosen = None;
        egui::ComboBox::from_id_salt("profile")
            .selected_text(current)
            .show_ui(ui, |ui| {
                for p in &self.profiles {
                    if ui
                        .selectable_label(Some(p.id) == self.state.profile, &p.name)
                        .clicked()
                    {
                        chosen = Some(p.id);
                    }
                }
            });
        if ui.small_button("Profiles…").clicked() {
            self.open_profiles();
        }
        if let Some(id) = chosen {
            self.state.profile = Some(id);
            self.series = None;
            self.reload_sources();
        }
    }

    fn source_picker(&mut self, ui: &mut egui::Ui) {
        let current = self
            .state
            .source
            .map_or("All sources", |id| self.source_name(id))
            .to_string();
        let mut chosen = None;
        egui::ComboBox::from_id_salt("source")
            .selected_text(current)
            .show_ui(ui, |ui| {
                if ui
                    .selectable_label(self.state.source.is_none(), "All sources")
                    .clicked()
                {
                    chosen = Some(None);
                }
                for s in &self.sources {
                    if ui
                        .selectable_label(self.state.source == Some(s.id), &s.name)
                        .clicked()
                    {
                        chosen = Some(Some(s.id));
                    }
                }
            });
        if let Some(src) = chosen {
            self.state.source = src;
            self.series = None;
            self.reload_categories();
        }
    }

    fn search_box(&mut self, ui: &mut egui::Ui) {
        let id = egui::Id::unique("search");
        if ui.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::F)) {
            ui.memory_mut(|m| m.request_focus(id));
        }
        let resp = ui.add(
            egui::TextEdit::singleline(&mut self.search.text)
                .id(id)
                .hint_text("Search (Ctrl+F)")
                .desired_width(220.0),
        );
        if resp.changed() {
            self.search.edited_at = Some(std::time::Instant::now());
        }
        if resp.has_focus() && ui.input(|i| i.key_pressed(Key::Escape)) {
            self.search.text.clear();
            self.search.edited_at = Some(std::time::Instant::now());
        }
    }
}
