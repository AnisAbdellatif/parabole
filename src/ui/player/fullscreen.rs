use super::App;

impl App {
    /// Draws the video over the whole window; returns false when not in fullscreen.
    pub(in crate::ui) fn embed_fullscreen(
        &mut self,
        ui: &mut egui::Ui,
        frame: &mut eframe::Frame,
    ) -> bool {
        if !self.player_ui.fullscreen {
            return false;
        }
        if !self.embedded_active() {
            let ctx = ui.ctx().clone();
            self.set_fullscreen(&ctx, false);
            return false;
        }
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ui, |ui| {
                let rect = ui.max_rect();
                self.video_surface(ui, frame, rect, true);
            });
        true
    }
}
