//! Title, meta, seek bar and control rows of the docked pane.

use super::controls::{Actions, fmt_time};
use super::session::Session;
use super::widgets::{BTN, bar, icon_button, track_menu};
use crate::model::MediaKind;
use crate::player::embed::EmbeddedPlayer;
use crate::player::{PlayerSettings, PlayerState, TrackKind};
use crate::ui::theme::{self, Icon};
use egui::{RichText, Ui};

pub fn title_meta(ui: &mut Ui, title: &str, meta: &str) {
    ui.add(
        egui::Label::new(
            RichText::new(title)
                .font(theme::semibold(16.0))
                .color(theme::TEXT),
        )
        .truncate(),
    );
    ui.add_space(4.0);
    ui.add(
        egui::Label::new(
            RichText::new(meta)
                .font(theme::regular(12.0))
                .color(theme::TEXT_MUTED),
        )
        .truncate(),
    );
}

pub fn hint(ui: &mut Ui, text: &str) {
    ui.add(egui::Label::new(
        RichText::new(text)
            .font(theme::regular(12.0))
            .color(theme::TEXT_MUTED),
    ));
}

fn meta_line(s: &Session, st: &PlayerState) -> String {
    let kind = match s.req.media {
        MediaKind::Live => "Live channel",
        MediaKind::Movie => "Movie",
        MediaKind::Series => "Episode",
    };
    match st.duration {
        Some(d) if !s.is_live() => format!("{kind} · {}", fmt_time(d)),
        _ => kind.to_string(),
    }
}

/// Title, meta and controls of the playing stream.
pub fn show(
    ui: &mut Ui,
    player: &mut EmbeddedPlayer,
    session: &Session,
    settings: &PlayerSettings,
    bottom: bool,
    stats_on: bool,
) -> Actions {
    let mut acts = Actions::default();
    let st = player.state().clone();
    title_meta(ui, &session.req.title, &meta_line(session, &st));
    ui.add_space(12.0);
    if let (false, Some(pos), Some(dur)) = (session.is_live(), st.time_pos, st.duration) {
        ui.horizontal(|ui| {
            let time = |t| {
                RichText::new(fmt_time(t))
                    .font(theme::regular(12.0))
                    .color(theme::TEXT_MUTED)
            };
            ui.label(time(pos));
            let w = ui.available_width() - 56.0;
            if let Some(f) = bar(ui, w, (pos / dur.max(1.0)) as f32) {
                player.seek_to(f64::from(f) * dur);
            }
            ui.label(time(dur));
        });
    }
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        let (play, tip) = if st.paused {
            (Icon::Play, "Play (Space)")
        } else {
            (Icon::Pause, "Pause (Space)")
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
        let w = ui.available_width() - 3.0 * (BTN + 4.0) - 4.0;
        if let Some(f) = bar(ui, w, (st.volume / 130.0) as f32) {
            player.set_volume(f64::from(f) * 130.0);
        }
        if icon_button(ui, Icon::Fullscreen, "Fullscreen (F)", false).clicked() {
            acts.toggle_fullscreen = true;
        }
        let (dock, tip) = if bottom {
            (Icon::DockRight, "Dock to the right")
        } else {
            (Icon::DockBottom, "Dock to the bottom")
        };
        if icon_button(ui, dock, tip, false).clicked() {
            acts.toggle_dock = true;
        }
        if icon_button(ui, Icon::Close, "Close player", false).clicked() {
            acts.close = true;
        }
    });
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        track_menu(ui, player, Icon::Subtitles, "Subtitles", TrackKind::Sub);
        track_menu(ui, player, Icon::Audio, "Audio track", TrackKind::Audio);
        if session.is_live() && icon_button(ui, Icon::Live, "Go live (Ctrl+L)", false).clicked() {
            let _ = player.reload(settings);
        }
        if icon_button(ui, Icon::Info, "Stats for nerds (I)", stats_on).clicked() {
            acts.toggle_stats = true;
        }
    });
    acts
}

/// Disabled controls shown while nothing plays in the embedded player.
pub fn idle_controls(ui: &mut Ui) {
    ui.add_enabled_ui(false, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            icon_button(ui, Icon::Play, "", false);
            icon_button(ui, Icon::Volume, "", false);
            let w = ui.available_width() - BTN - 4.0;
            bar(ui, w, 0.6);
            icon_button(ui, Icon::Fullscreen, "", false);
        });
    });
}
