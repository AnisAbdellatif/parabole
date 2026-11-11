use super::{App, dock, widgets};
use crate::ui::theme;
use egui::{Rect, Sense, Stroke, StrokeKind, Ui, vec2};

impl App {
    /// Resizable player panel, docked right or at the bottom; must run before the central panel.
    pub(in crate::ui) fn embed_pane(&mut self, ui: &mut Ui, frame: &mut eframe::Frame) {
        let bottom = self.state.player_dock_bottom;
        let panel = if bottom {
            egui::Panel::bottom("player").default_size(300.0)
        } else {
            egui::Panel::right("player").default_size(theme::PLAYER_W)
        };
        let frame_style = egui::Frame::new().fill(theme::PANEL).inner_margin(16);
        panel
            .resizable(true)
            .min_size(240.0)
            .frame(frame_style)
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = vec2(4.0, 0.0);
                if bottom {
                    let h = ui.available_height();
                    let w = (h * 16.0 / 9.0).min(ui.available_width() * 0.5);
                    ui.horizontal_top(|ui| {
                        let rect = Rect::from_min_size(ui.cursor().min, vec2(w, h));
                        self.video_box(ui, frame, rect);
                        ui.add_space(12.0);
                        ui.vertical(|ui| self.pane_info(ui, bottom));
                    });
                } else {
                    let w = ui.available_width();
                    let rect = Rect::from_min_size(ui.cursor().min, vec2(w, w * 9.0 / 16.0));
                    self.video_box(ui, frame, rect);
                    ui.add_space(16.0);
                    self.pane_info(ui, bottom);
                }
            });
    }

    fn video_box(&mut self, ui: &mut Ui, frame: &mut eframe::Frame, rect: Rect) {
        let active = self.embedded_active();
        let painter = ui.painter().clone();
        let fill = if active {
            egui::Color32::BLACK
        } else {
            theme::VIDEO_IDLE
        };
        painter.rect_filled(rect, theme::RADIUS_CARD, fill);
        if active {
            self.video_surface(ui, frame, rect, false);
        } else {
            ui.allocate_rect(rect, Sense::hover());
            let c = rect.center();
            painter.circle_filled(c, 28.0, theme::ACCENT);
            theme::icon(theme::Icon::Play, 24.0, theme::ON_ACCENT)
                .paint_at(ui, Rect::from_center_size(c, vec2(24.0, 24.0)));
        }
        painter.rect_stroke(
            rect,
            theme::RADIUS_CARD,
            Stroke::new(1.0, theme::DIVIDER),
            StrokeKind::Inside,
        );
        let (text, bg, fg) = match (active, self.live_playing(), self.playing.is_some()) {
            (_, true, _) => ("LIVE", theme::LIVE_BG, theme::LIVE_FG),
            (true, false, _) | (false, false, true) => ("VOD", theme::INFO_BG, theme::INFO_FG),
            _ => ("IDLE", theme::SURFACE, theme::TEXT_MUTED),
        };
        widgets::tag(&painter, rect.min + vec2(12.0, 10.0), text, bg, fg);
    }

    fn pane_info(&mut self, ui: &mut Ui, bottom: bool) {
        let ctx = ui.ctx().clone();
        let ui_state = &mut self.player_ui;
        let acts = match (ui_state.embed.as_mut(), ui_state.session.as_ref()) {
            (Some(player), Some(session)) => {
                let acts = dock::show(
                    ui,
                    player,
                    session,
                    &self.player_settings,
                    bottom,
                    ui_state.stats,
                );
                ui.add_space(12.0);
                let live = if session.is_live() {
                    " · Ctrl+L go live"
                } else {
                    ""
                };
                separator_and_hint(
                    ui,
                    &format!(
                        "Space pause · F fullscreen · PgUp/PgDn change channel · I stats{live}"
                    ),
                );
                acts
            }
            _ => {
                let (title, meta) = match &self.playing {
                    Some((title, _)) => (title.as_str(), "Playing in an external player"),
                    None => ("Nothing playing", "Idle"),
                };
                dock::title_meta(ui, title, meta);
                ui.add_space(16.0);
                dock::idle_controls(ui);
                ui.add_space(12.0);
                separator_and_hint(
                    ui,
                    "Pick a channel, movie or episode to start playback. The player can dock right or at the bottom.",
                );
                return;
            }
        };
        if acts.toggle_stats {
            self.player_ui.stats = !self.player_ui.stats;
        }
        if acts.toggle_dock {
            self.state.player_dock_bottom = !bottom;
        }
        if acts.toggle_mini {
            self.set_mini(&ctx, true);
        }
        if acts.toggle_fullscreen {
            self.set_fullscreen(&ctx, true);
        }
        if acts.close {
            self.stop_embedded(&ctx);
        }
    }
}

fn separator_and_hint(ui: &mut Ui, text: &str) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), Sense::hover());
    ui.painter().rect_filled(rect, 0.0, theme::DIVIDER);
    ui.add_space(14.0);
    dock::hint(ui, text);
}
