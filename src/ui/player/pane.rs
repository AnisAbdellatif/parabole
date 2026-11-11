use super::App;
use egui::{Align, Layout};

impl App {
    /// Resizable player panel, docked right or bottom; must run before the central panel.
    pub(in crate::ui) fn embed_pane(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let Some(title) = self.player_ui.session.as_ref().map(|s| s.req.title.clone()) else {
            return;
        };
        let bottom = self.state.player_dock_bottom;
        let panel = if bottom {
            egui::Panel::bottom("player").default_size(320.0)
        } else {
            egui::Panel::right("player").default_size(520.0)
        };
        panel.resizable(true).min_size(200.0).show(ui, |ui| {
            let ctx = ui.ctx().clone();
            ui.horizontal(|ui| {
                ui.strong(egui::RichText::new(title).size(14.0));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.small_button("✕").on_hover_text("Close player").clicked() {
                        self.stop_embedded(&ctx);
                    }
                    let (label, tip) = if bottom {
                        ("⇥", "Dock to the right")
                    } else {
                        ("⇩", "Dock to the bottom")
                    };
                    if ui.small_button(label).on_hover_text(tip).clicked() {
                        self.state.player_dock_bottom = !bottom;
                    }
                });
            });
            let rect = ui.available_rect_before_wrap();
            self.video_surface(ui, frame, rect);
        });
    }
}
