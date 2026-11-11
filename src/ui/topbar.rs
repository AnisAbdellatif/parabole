use super::App;
use super::theme;
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
        let frame = egui::Frame::new()
            .fill(theme::PANEL)
            .stroke(egui::Stroke::new(1.0, theme::DIVIDER))
            .inner_margin(egui::Margin::symmetric(16, 0));
        egui::Panel::top("topbar")
            .exact_size(theme::TOPBAR_H)
            .frame(frame)
            .show(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.spacing_mut().item_spacing.x = 16.0;
                    self.profile_picker(ui);
                    divider(ui);
                    self.source_picker(ui);
                    self.tabs(ui);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(egui::Button::image_and_text(
                                theme::icon(theme::Icon::Settings, 15.0, theme::TEXT),
                                "Settings…",
                            ))
                            .clicked()
                        {
                            self.open_settings();
                        }
                        if ui.button("Sources…").clicked() {
                            self.open_sources();
                        }
                        self.search_box(ui);
                    });
                });
            });
    }

    fn tabs(&mut self, ui: &mut egui::Ui) {
        egui::Frame::new()
            .fill(theme::SURFACE)
            .stroke(egui::Stroke::new(1.0, theme::BORDER))
            .corner_radius(theme::RADIUS_L)
            .inner_margin(egui::Margin::same(3))
            .show(ui, |ui| {
                // horizontal_centered children otherwise stretch to the full bar height
                ui.set_height(30.0);
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.spacing_mut().interact_size.y = 30.0;
                for kind in MediaKind::ALL {
                    let on = self.state.tab == kind;
                    let text = if on {
                        egui::RichText::new(tab_label(kind))
                            .font(theme::semibold(13.0))
                            .color(theme::ON_ACCENT)
                    } else {
                        egui::RichText::new(tab_label(kind))
                            .font(theme::semibold(13.0))
                            .color(theme::TEXT_MUTED)
                    };
                    let fill = if on {
                        theme::ACCENT
                    } else {
                        egui::Color32::TRANSPARENT
                    };
                    let btn = egui::Button::new(text)
                        .fill(fill)
                        .stroke(egui::Stroke::NONE)
                        .corner_radius(7)
                        .min_size(egui::vec2(0.0, 30.0));
                    if ui.add(btn).clicked() {
                        self.set_tab(kind);
                    }
                }
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
        let label = egui::RichText::new("Profiles…").color(theme::TEXT_MUTED);
        if ui.button(label).clicked() {
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
        let avail = ui.available_width();
        let outer = avail.clamp(146.0, 460.0);
        ui.add_space(((avail - outer) / 2.0).max(0.0));
        let width = outer - 26.0;
        let mut resp = None;
        egui::Frame::new()
            .fill(theme::INPUT_BG)
            .stroke(egui::Stroke::new(1.0, theme::BORDER))
            .corner_radius(theme::RADIUS_L)
            .inner_margin(egui::Margin::symmetric(12, 0))
            .show(ui, |ui| {
                ui.set_height(38.0);
                ui.set_width(width);
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.spacing_mut().interact_size.y = 18.0;
                    ui.add(theme::icon(theme::Icon::Search, 16.0, theme::TEXT_MUTED));
                    resp = Some(
                        ui.add(
                            egui::TextEdit::singleline(&mut self.search.text)
                                .id(id)
                                .frame(egui::Frame::NONE)
                                .hint_text("Search (Ctrl+F)")
                                .desired_width(ui.available_width() - 48.0),
                        ),
                    );
                    egui::Frame::new()
                        .stroke(egui::Stroke::new(1.0, theme::BORDER))
                        .corner_radius(5)
                        .inner_margin(egui::Margin::symmetric(6, 1))
                        .show(ui, |ui| {
                            ui.set_height(16.0);
                            ui.label(
                                egui::RichText::new("Esc")
                                    .size(11.0)
                                    .color(theme::TEXT_MUTED),
                            );
                        });
                });
            });
        let Some(resp) = resp else {
            return;
        };
        if resp.changed() {
            self.search.edited_at = Some(std::time::Instant::now());
        }
        if resp.has_focus() && ui.input(|i| i.key_pressed(Key::Escape)) {
            self.search.text.clear();
            self.search.edited_at = Some(std::time::Instant::now());
        }
    }
}

fn divider(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(1.0, 24.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 0.0, theme::BORDER);
}
