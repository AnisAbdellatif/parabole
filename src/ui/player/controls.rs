use super::session::Session;
use crate::player::embed::EmbeddedPlayer;
use crate::player::{PlayerSettings, TrackKind};
use egui::{Align, Color32, Layout, Rect, Slider, pos2};

const BAR_HEIGHT: f32 = 40.0;

#[derive(Default)]
pub struct Actions {
    pub toggle_fullscreen: bool,
    pub close: bool,
}

pub fn fmt_time(seconds: f64) -> String {
    let s = seconds.max(0.0) as u64;
    match s / 3600 {
        0 => format!("{}:{:02}", s / 60, s % 60),
        h => format!("{h}:{:02}:{:02}", s / 60 % 60, s % 60),
    }
}

/// Overlay bar at the bottom of `rect`; `alpha` 0 hides it.
pub fn show(
    ui: &mut egui::Ui,
    rect: Rect,
    player: &mut EmbeddedPlayer,
    session: &Session,
    settings: &PlayerSettings,
    alpha: f32,
) -> Actions {
    let mut acts = Actions::default();
    if alpha < 0.02 {
        return acts;
    }
    let bar = Rect::from_min_max(pos2(rect.left(), rect.bottom() - BAR_HEIGHT), rect.max);
    ui.painter()
        .rect_filled(bar, 0.0, Color32::from_black_alpha((170.0 * alpha) as u8));
    let st = player.state().clone();
    ui.scope_builder(
        egui::UiBuilder::new().max_rect(bar.shrink2(egui::vec2(8.0, 4.0))),
        |ui| {
            ui.set_opacity(alpha);
            ui.horizontal_centered(|ui| {
                if ui.button(if st.paused { "▶" } else { "⏸" }).clicked() {
                    player.set_paused(!st.paused);
                }
                if ui.button("■").on_hover_text("Stop").clicked() {
                    acts.close = true;
                }
                if ui.button(if st.muted { "🔇" } else { "🔊" }).clicked() {
                    player.set_muted(!st.muted);
                }
                let mut volume = st.volume;
                ui.spacing_mut().slider_width = 70.0;
                if ui
                    .add(Slider::new(&mut volume, 0.0..=130.0).show_value(false))
                    .changed()
                {
                    player.set_volume(volume);
                }
                if let (false, Some(pos), Some(dur)) = (session.is_live(), st.time_pos, st.duration)
                {
                    ui.label(fmt_time(pos));
                    let mut at = pos;
                    ui.spacing_mut().slider_width = (rect.width() - 520.0).max(60.0);
                    if ui
                        .add(Slider::new(&mut at, 0.0..=dur.max(1.0)).show_value(false))
                        .changed()
                    {
                        player.seek_to(at);
                    }
                    ui.label(fmt_time(dur));
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.button("Fullscreen").clicked() {
                        acts.toggle_fullscreen = true;
                    }
                    if session.is_live() && ui.button("Go live").on_hover_text("Ctrl+L").clicked() {
                        let _ = player.reload(settings);
                    }
                    track_menu(ui, player, "Subs", TrackKind::Sub, true);
                    track_menu(ui, player, "Audio", TrackKind::Audio, false);
                });
            });
        },
    );
    acts
}

fn track_menu(
    ui: &mut egui::Ui,
    player: &mut EmbeddedPlayer,
    label: &str,
    kind: TrackKind,
    off: bool,
) {
    let tracks: Vec<_> = player
        .state()
        .tracks
        .iter()
        .filter(|t| t.kind == kind)
        .cloned()
        .collect();
    if tracks.is_empty() {
        return;
    }
    ui.menu_button(format!("{label} ▾"), |ui| {
        if off
            && ui
                .selectable_label(!tracks.iter().any(|t| t.selected), "Off")
                .clicked()
        {
            player.select_track(kind, None);
            ui.close();
        }
        for t in &tracks {
            if ui.selectable_label(t.selected, t.label()).clicked() {
                player.select_track(kind, Some(t.id));
                ui.close();
            }
        }
    });
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
