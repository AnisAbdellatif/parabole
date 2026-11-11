use crate::player::TrackKind;
use crate::player::embed::EmbeddedPlayer;
use crate::ui::theme::{self, Icon};
use egui::{Color32, Painter, Pos2, Rect, Response, Sense, Ui, vec2};

pub const BTN: f32 = 32.0;

/// Square icon button; `on` tints it with the accent.
pub fn icon_button(ui: &mut Ui, ic: Icon, tip: &str, on: bool) -> Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(BTN, BTN), Sense::click());
    let enabled = ui.is_enabled();
    if enabled && resp.hovered() {
        ui.painter()
            .rect_filled(rect, theme::RADIUS, theme::SURFACE_HOVER);
    }
    let tint = if !enabled {
        theme::TEXT_FAINT
    } else if on {
        theme::ACCENT
    } else if resp.hovered() {
        theme::TEXT
    } else {
        theme::TEXT_MUTED
    };
    theme::icon(ic, 20.0, tint)
        .paint_at(ui, Rect::from_center_size(rect.center(), vec2(20.0, 20.0)));
    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
    if tip.is_empty() {
        resp
    } else {
        resp.on_hover_text(tip)
    }
}

/// Thin accent progress bar; returns the new fraction (0..=1) while clicked or dragged.
pub fn bar(ui: &mut Ui, width: f32, frac: f32) -> Option<f32> {
    let (rect, resp) = ui.allocate_exact_size(vec2(width.max(20.0), 16.0), Sense::click_and_drag());
    let track = Rect::from_center_size(rect.center(), vec2(rect.width(), 4.0));
    let mut fill = track;
    fill.set_right(track.left() + track.width() * frac.clamp(0.0, 1.0));
    let p = ui.painter();
    p.rect_filled(track, 2.0, theme::BORDER);
    p.rect_filled(fill, 2.0, theme::ACCENT);
    if resp.hovered() || resp.dragged() {
        p.circle_filled(Pos2::new(fill.right(), track.center().y), 5.0, theme::TEXT);
    }
    let x = ui.input(|i| i.pointer.interact_pos())?.x;
    (resp.dragged() || resp.clicked()).then(|| ((x - track.left()) / track.width()).clamp(0.0, 1.0))
}

pub fn tag(painter: &Painter, pos: Pos2, text: &str, bg: Color32, fg: Color32) {
    let galley = painter.layout_no_wrap(text.into(), theme::semibold(11.0), fg);
    let rect = Rect::from_min_size(pos, galley.size() + vec2(16.0, 6.0));
    painter.rect_filled(rect, 6.0, bg);
    painter.galley(rect.min + vec2(8.0, 3.0), galley, fg);
}

/// Icon button opening a popup of the player's tracks of `kind`; hidden when there are none.
pub fn track_menu(ui: &mut Ui, player: &mut EmbeddedPlayer, ic: Icon, tip: &str, kind: TrackKind) {
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
    let off = kind == TrackKind::Sub;
    let resp = icon_button(ui, ic, tip, off && tracks.iter().any(|t| t.selected));
    egui::Popup::menu(&resp).show(|ui| {
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
