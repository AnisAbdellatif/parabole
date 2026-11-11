use super::{App, PlayerUi, controls, nerd_stats};
use crate::ui::theme;
use egui::{Align2, Color32, Rect, Vec2, vec2};
use std::time::{Duration, Instant};

const FADE_AFTER_S: f32 = 3.0;
const FADE_FOR_S: f32 = 0.4;

impl App {
    /// Video, buffering/error overlays and the fading controls, shared by the pane and fullscreen.
    pub(super) fn video_surface(
        &mut self,
        ui: &mut egui::Ui,
        frame: &mut eframe::Frame,
        rect: Rect,
        overlay: bool,
    ) {
        let ctx = ui.ctx().clone();
        let resp = ui.allocate_rect(rect, egui::Sense::click());
        if overlay {
            ui.painter().rect_filled(rect, 0.0, Color32::BLACK);
        }
        let PlayerUi {
            embed,
            session,
            last_motion,
            video_rect,
            fullscreen,
            mini,
            zap,
            stats,
            ..
        } = &mut self.player_ui;
        let (Some(player), Some(s)) = (embed.as_mut(), session.as_mut()) else {
            return;
        };
        *video_rect = rect;
        player.paint(ui, frame, rect);
        if *stats {
            nerd_stats::paint(ui.painter(), rect, &player.stats());
            ctx.request_repaint_after(Duration::from_millis(500));
        }

        if let Some(text) = zap.text() {
            zap_overlay(ui.painter(), rect, text);
        }

        if resp.hovered()
            && ctx.input(|i| i.pointer.delta() != Vec2::ZERO || i.pointer.any_pressed())
        {
            *last_motion = Instant::now();
        }
        let idle = last_motion.elapsed().as_secs_f32();
        let sticky = player.state().paused || s.error.is_some() || egui::Popup::is_any_open(&ctx);
        let alpha = if sticky {
            1.0
        } else {
            (1.0 - (idle - FADE_AFTER_S) / FADE_FOR_S).clamp(0.0, 1.0)
        };
        if !sticky && alpha > 0.0 {
            let wait = if idle < FADE_AFTER_S {
                FADE_AFTER_S - idle
            } else {
                0.03
            };
            ctx.request_repaint_after(Duration::from_secs_f32(wait + 0.02));
        }
        if *fullscreen && alpha == 0.0 && resp.hovered() {
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::None);
        }

        if let Some(err) = &s.error {
            ui.put(
                rect,
                egui::Label::new(egui::RichText::new(err).color(Color32::LIGHT_RED)),
            );
        } else if player.state().buffering
            || s.retry_at.is_some()
            || player.state().time_pos.is_none()
        {
            ui.put(
                Rect::from_center_size(rect.center(), Vec2::splat(32.0)),
                egui::Spinner::new().size(32.0),
            );
        }

        let mut acts = if overlay {
            controls::show(ui, rect, player, s, &self.player_settings, alpha, *stats)
        } else {
            controls::Actions::default()
        };
        if *mini {
            acts.toggle_mini |= resp.double_clicked();
        } else {
            acts.toggle_fullscreen |= resp.double_clicked();
        }
        if acts.toggle_stats {
            *stats = !*stats;
        }
        let (toggle, close, on, mini_on) =
            (acts.toggle_fullscreen, acts.close, !*fullscreen, *mini);
        if toggle {
            self.set_fullscreen(&ctx, on);
        }
        if acts.toggle_mini {
            self.set_mini(&ctx, !mini_on);
        }
        if close {
            self.stop_embedded(&ctx);
        }
    }
}

fn zap_overlay(painter: &egui::Painter, rect: Rect, text: &str) {
    let font = theme::semibold(56.0);
    let at = rect.right_top() + vec2(-20.0, 16.0);
    painter.text(
        at + vec2(2.0, 2.0),
        Align2::RIGHT_TOP,
        text,
        font.clone(),
        Color32::BLACK,
    );
    painter.text(at, Align2::RIGHT_TOP, text, font, Color32::WHITE);
}
