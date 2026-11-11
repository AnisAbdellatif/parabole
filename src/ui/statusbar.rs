use super::App;
use super::theme::{self, TEXT_MUTED};
use crate::player::PlayerKind;
use crate::worker::Command;
use egui::RichText;

fn kind_name(kind: &PlayerKind) -> &'static str {
    match kind {
        PlayerKind::Embedded => "the built-in player",
        PlayerKind::Mpv => "mpv",
        PlayerKind::Vlc => "VLC",
        PlayerKind::Custom => "custom player",
    }
}

impl App {
    pub(super) fn status_bar(&mut self, ui: &mut egui::Ui) {
        let frame = egui::Frame::new()
            .fill(theme::PANEL)
            .stroke(egui::Stroke::new(1.0, theme::DIVIDER))
            .inner_margin(egui::Margin::symmetric(16, 0));
        egui::Panel::bottom("status")
            .exact_size(theme::STATUSBAR_H)
            .frame(frame)
            .show(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.spacing_mut().item_spacing.x = 16.0;
                    ui.style_mut().override_text_style = Some(egui::TextStyle::Small);
                    self.status_left(ui);
                    self.playing_controls(ui);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(self.profile_label()).color(TEXT_MUTED));
                    });
                });
            });
    }

    fn profile_label(&self) -> String {
        let name = self
            .profiles
            .iter()
            .find(|p| Some(p.id) == self.state.profile)
            .map_or("?", |p| p.name.as_str());
        format!("Profile: {name}")
    }

    fn status_left(&mut self, ui: &mut egui::Ui) {
        let syncing = !self.refreshing.is_empty();
        let (dot, text) = if syncing {
            (theme::WARN, "Syncing…".to_string())
        } else {
            let n = self.sources.len();
            (theme::OK, format!("{n} source{} synced", plural(n)))
        };
        let (rect, _) = ui.allocate_exact_size(egui::vec2(7.0, 7.0), egui::Sense::hover());
        ui.painter().circle_filled(rect.center(), 3.5, dot);
        ui.add_space(-10.0);
        ui.label(RichText::new(text).color(TEXT_MUTED));
        let items = format!(
            "{} items in {} categories",
            self.streams.len(),
            self.categories.len()
        );
        ui.label(RichText::new(items).color(TEXT_MUTED));
        if let Some((title, kind)) = &self.playing {
            ui.label(
                RichText::new(format!("Playing: {title} in {}", kind_name(kind)))
                    .color(theme::TEXT),
            );
        }
    }

    fn playing_controls(&mut self, ui: &mut egui::Ui) {
        let Some((_, kind)) = self.playing.clone() else {
            return;
        };
        ui.spacing_mut().interact_size.y = 22.0;
        ui.spacing_mut().button_padding = egui::vec2(8.0, 2.0);
        if self.live_playing() && ui.button("Go live").on_hover_text("Ctrl+L").clicked() {
            self.go_live();
        }
        if ui.button("■ Stop").clicked() {
            if kind == PlayerKind::Embedded {
                self.stop_embedded(ui.ctx());
            } else {
                self.worker.send(Command::StopPlayer);
            }
            self.playing = None;
        }
    }
}

fn plural(n: usize) -> &'static str {
    if n == 1 { "" } else { "s" }
}
