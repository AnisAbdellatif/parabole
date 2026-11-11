use super::App;
use super::theme;
use std::time::Duration;

const TOAST_TTL: Duration = Duration::from_secs(8);

impl App {
    pub(super) fn toasts(&mut self, ctx: &egui::Context) {
        self.toasts.retain(|t| t.born.elapsed() < TOAST_TTL);
        if self.toasts.is_empty() {
            return;
        }
        ctx.request_repaint_after(Duration::from_secs(1));
        let mut dismiss = None;
        egui::Area::new(egui::Id::unique("toasts"))
            .anchor(egui::Align2::RIGHT_BOTTOM, [-16.0, -46.0])
            .show(ctx, |ui| {
                for (i, t) in self.toasts.iter().enumerate() {
                    egui::Frame::new()
                        .fill(theme::TOAST_BG)
                        .stroke(egui::Stroke::new(1.0, theme::TOAST_BORDER))
                        .corner_radius(theme::RADIUS_L)
                        .inner_margin(egui::Margin::symmetric(14, 12))
                        .shadow(egui::Shadow {
                            offset: [0, 8],
                            blur: 24,
                            spread: 0,
                            color: egui::Color32::from_black_alpha(100),
                        })
                        .show(ui, |ui| {
                            ui.set_max_width(300.0);
                            ui.horizontal_wrapped(|ui| {
                                ui.label(egui::RichText::new(&t.text).color(theme::TEXT));
                                if theme::icon_button(ui, theme::Icon::Close, "Dismiss").clicked() {
                                    dismiss = Some(i);
                                }
                            });
                        });
                }
            });
        if let Some(i) = dismiss {
            self.toasts.remove(i);
        }
    }
}
