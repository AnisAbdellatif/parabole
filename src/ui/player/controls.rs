use super::session::Session;
use super::widgets::{bar, icon_button, track_menu};
use crate::player::embed::EmbeddedPlayer;
use crate::player::{PlayerSettings, TrackKind};
use crate::ui::theme::{self, Icon};
use egui::{Align, Color32, Layout, Rect, pos2};

const BAR_HEIGHT: f32 = 48.0;

#[derive(Default)]
pub struct Actions {
    pub toggle_fullscreen: bool,
    pub close: bool,
    pub toggle_dock: bool,
    pub toggle_stats: bool,
}

pub fn fmt_time(seconds: f64) -> String {
    let s = seconds.max(0.0) as u64;
    match s / 3600 {
        0 => format!("{}:{:02}", s / 60, s % 60),
        h => format!("{h}:{:02}:{:02}", s / 60 % 60, s % 60),
    }
}

/// Fullscreen overlay bar at the bottom of `rect`; `alpha` 0 hides it.
pub fn show(
    ui: &mut egui::Ui,
    rect: Rect,
    player: &mut EmbeddedPlayer,
    session: &Session,
    settings: &PlayerSettings,
    alpha: f32,
    stats_on: bool,
) -> Actions {
    let mut acts = Actions::default();
    if alpha < 0.02 {
        return acts;
    }
    let bar_rect = Rect::from_min_max(pos2(rect.left(), rect.bottom() - BAR_HEIGHT), rect.max);
    ui.painter().rect_filled(
        bar_rect,
        0.0,
        Color32::from_black_alpha((190.0 * alpha) as u8),
    );
    let st = player.state().clone();
    ui.scope_builder(
        egui::UiBuilder::new().max_rect(bar_rect.shrink2(egui::vec2(12.0, 4.0))),
        |ui| {
            ui.set_opacity(alpha);
            ui.horizontal_centered(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                let (play, tip) = if st.paused {
                    (Icon::Play, "Play")
                } else {
                    (Icon::Pause, "Pause")
                };
                if icon_button(ui, play, tip, false).clicked() {
                    player.set_paused(!st.paused);
                }
                if icon_button(ui, Icon::Stop, "Stop", false).clicked() {
                    acts.close = true;
                }
                let (vol, tip) = if st.muted {
                    (Icon::VolumeOff, "Unmute")
                } else {
                    (Icon::Volume, "Mute")
                };
                if icon_button(ui, vol, tip, false).clicked() {
                    player.set_muted(!st.muted);
                }
                if let Some(f) = bar(ui, 80.0, (st.volume / 130.0) as f32) {
                    player.set_volume(f64::from(f) * 130.0);
                }
                if let (false, Some(pos), Some(dur)) = (session.is_live(), st.time_pos, st.duration)
                {
                    let time = |t| {
                        egui::RichText::new(fmt_time(t))
                            .font(theme::regular(12.0))
                            .color(theme::TEXT_MUTED)
                    };
                    ui.label(time(pos));
                    if let Some(f) = bar(ui, rect.width() - 520.0, (pos / dur.max(1.0)) as f32) {
                        player.seek_to(f64::from(f) * dur);
                    }
                    ui.label(time(dur));
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    if icon_button(ui, Icon::FullscreenExit, "Exit fullscreen (Esc)", false)
                        .clicked()
                    {
                        acts.toggle_fullscreen = true;
                    }
                    if icon_button(ui, Icon::Info, "Stats for nerds (I)", stats_on).clicked() {
                        acts.toggle_stats = true;
                    }
                    if session.is_live()
                        && icon_button(ui, Icon::Live, "Go live (Ctrl+L)", false).clicked()
                    {
                        let _ = player.reload(settings);
                    }
                    track_menu(ui, player, Icon::Subtitles, "Subtitles", TrackKind::Sub);
                    track_menu(ui, player, Icon::Audio, "Audio track", TrackKind::Audio);
                });
            });
        },
    );
    acts
}

#[cfg(test)]
mod tests {
    use super::fmt_time;

    #[test]
    fn formats_minutes_and_hours() {
        assert_eq!(fmt_time(65.4), "1:05");
        assert_eq!(fmt_time(3725.0), "1:02:05");
        assert_eq!(fmt_time(-3.0), "0:00");
    }
}
