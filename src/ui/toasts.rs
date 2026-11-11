use super::App;
use crate::player::PlayerKind;
use crate::worker::Command;
use std::time::Duration;

const TOAST_TTL: Duration = Duration::from_secs(8);

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
        let ctx = ui.ctx().clone();
        egui::Panel::bottom("status").show(ui, |ui| {
            ui.horizontal(|ui| match &self.playing {
                Some((title, kind)) => {
                    ui.label(format!("Playing: {title} in {}", kind_name(kind)));
                    if ui.button("■ Stop").clicked() {
                        if *kind == PlayerKind::Embedded {
                            self.stop_embedded(&ctx);
                        } else {
                            self.worker.send(Command::StopPlayer);
                        }
                        self.playing = None;
                    }
                    if self.live_playing() && ui.button("Go live").on_hover_text("Ctrl+L").clicked()
                    {
                        self.go_live();
                    }
                }
                None => {
                    ui.weak("Idle");
                }
            });
        });
    }

    pub(super) fn toasts(&mut self, ctx: &egui::Context) {
        self.toasts.retain(|t| t.born.elapsed() < TOAST_TTL);
        if self.toasts.is_empty() {
            return;
        }
        ctx.request_repaint_after(Duration::from_secs(1));
        let mut dismiss = None;
        egui::Area::new(egui::Id::unique("toasts"))
            .anchor(egui::Align2::RIGHT_BOTTOM, [-12.0, -40.0])
            .show(ctx, |ui| {
                for (i, t) in self.toasts.iter().enumerate() {
                    egui::Frame::popup(ui.style()).show(ui, |ui| {
                        ui.set_max_width(420.0);
                        ui.horizontal_wrapped(|ui| {
                            ui.colored_label(egui::Color32::LIGHT_RED, &t.text);
                            if ui.small_button("✕").clicked() {
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
