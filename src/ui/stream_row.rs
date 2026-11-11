//! One 62px Live channel row.
use super::kit;
use super::progress_marks;
use super::row_menu;
use super::theme::{self, Icon};
use super::tile;
use crate::model::StreamRow;
use crate::player::PlayerKind;
use egui::{Color32, CornerRadius, Rect, Sense, Stroke, StrokeKind, UiBuilder, pos2, vec2};

pub const ROW_H: f32 = theme::CHANNEL_ROW_H;

pub enum RowAction {
    Select,
    Activate,
    Play(PlayerKind),
    ToggleFavorite,
    ToggleHidden,
    CopyUrl,
    Info,
    AddToGroup(i64),
    NewGroup,
    RemoveFromGroup,
    PlayFrom(i64),
    LoadAlternates,
}

/// One list row; `cursor` is the keyboard selection, `playing` the channel in the player.
pub fn show(
    ui: &mut egui::Ui,
    row: &StreamRow,
    cursor: bool,
    playing: bool,
    edit: bool,
) -> Vec<RowAction> {
    let mut out = Vec::new();
    let rect = ui
        .allocate_exact_size(vec2(ui.available_width(), ROW_H), Sense::hover())
        .0;
    let mut ui = ui.new_child(UiBuilder::new().max_rect(rect));
    if row.hidden || row.missing {
        ui.set_opacity(0.5);
    }
    let resp = ui.interact(rect, ui.scope_id().with(("row", row.id)), Sense::click());
    paint_bg(&ui, rect, cursor, playing);

    let cy = rect.center().y;
    let logo = Rect::from_min_size(
        pos2(rect.left() + 10.0, cy - theme::LOGO / 2.0),
        vec2(theme::LOGO, theme::LOGO),
    );
    tile::logo_tile(&ui, logo, &row.name, row.logo.as_deref());

    let button = |right: f32| Rect::from_min_size(pos2(right - 32.0, cy - 16.0), vec2(32.0, 32.0));
    let star_rect = button(rect.right() - 6.0);
    let (ic, tint, tip) = if row.favorite {
        (Icon::StarFilled, theme::STAR, "Remove favorite")
    } else {
        (Icon::Star, theme::TEXT_MUTED, "Add favorite")
    };
    let base = ui.scope_id().with(row.id);
    if kit::icon_btn(&mut ui, star_rect, base.with("fav"), ic, 18.0, tint, tip).clicked() {
        out.push(RowAction::ToggleFavorite);
    }
    let mut text_right = star_rect.left() - 8.0;
    if progress_marks::is_watched(row.progress) {
        progress_marks::paint_check(&ui, pos2(text_right - 9.0, cy));
        text_right -= 26.0;
    }
    progress_marks::paint_bar(&ui, rect.shrink2(vec2(10.0, 0.0)), row.progress);
    if edit {
        let eye_rect = button(star_rect.left() - 4.0);
        let (ic, tip) = if row.hidden {
            (Icon::EyeOff, "Show")
        } else {
            (Icon::Eye, "Hide")
        };
        if kit::icon_btn(
            &mut ui,
            eye_rect,
            base.with("eye"),
            ic,
            16.0,
            theme::TEXT_MUTED,
            tip,
        )
        .clicked()
        {
            out.push(RowAction::ToggleHidden);
        }
        text_right = eye_rect.left() - 8.0;
    }

    let left = logo.right() + 14.0;
    let meta = meta_line(row);
    let name_y = if meta.is_empty() { cy } else { cy - 9.0 };
    kit::paint_text(
        &ui,
        left,
        name_y,
        text_right - left,
        &row.name,
        theme::medium(14.0),
        theme::TEXT,
    );
    if !meta.is_empty() {
        kit::paint_text(
            &ui,
            left,
            cy + 10.0,
            text_right - left,
            &meta,
            theme::regular(12.0),
            theme::TEXT_MUTED,
        );
    }

    if resp.clicked() {
        out.push(RowAction::Select);
    }
    if resp.double_clicked() && !row.missing {
        out.push(RowAction::Activate);
    }
    resp.context_menu(|ui| row_menu::show(ui, row, &mut out));
    out
}

fn meta_line(row: &StreamRow) -> String {
    if row.missing {
        "No longer listed by the provider".to_string()
    } else {
        row.num.map(|n| format!("Channel {n}")).unwrap_or_default()
    }
}

fn paint_bg(ui: &egui::Ui, rect: Rect, cursor: bool, playing: bool) {
    let r = CornerRadius::same(theme::RADIUS_L);
    let (fill, border) = if playing {
        (theme::SELECTED_BG, theme::SELECTED_BORDER)
    } else if cursor {
        (theme::ROW_HOVER, theme::BORDER)
    } else if ui.rect_contains_pointer(rect) {
        (theme::ROW_HOVER, Color32::TRANSPARENT)
    } else {
        return;
    };
    ui.painter().rect_filled(rect, r, fill);
    ui.painter()
        .rect_stroke(rect, r, Stroke::new(1.0, border), StrokeKind::Inside);
}
