//! Movies/Series poster cards laid out in rows of a responsive grid.
use super::kit;
use super::progress_marks;
use super::row_menu;
use super::stream_row::RowAction;
use super::theme::{self, Icon};
use super::tile::{self, Fit};
use crate::model::StreamRow;
use egui::text::{LayoutJob, TextWrapping};
use egui::{
    Align2, Color32, CornerRadius, Rect, Sense, Stroke, StrokeKind, Ui, UiBuilder, pos2, vec2,
};

const TITLE_H: f32 = 58.0;

/// Columns and card width that fill `avail` with cards at least POSTER_MIN_W wide.
pub fn columns(avail: f32) -> (usize, f32) {
    let cols =
        (((avail + theme::GAP) / (theme::POSTER_MIN_W + theme::GAP)).floor() as usize).max(1);
    let width = (avail - theme::GAP * (cols - 1) as f32) / cols as f32;
    (cols, width)
}

pub fn card_height(card_w: f32) -> f32 {
    card_w * 1.5 + TITLE_H
}

pub struct RowCtx<'a> {
    pub first_index: usize,
    pub card_w: f32,
    pub cursor: usize,
    pub playing: Option<&'a str>,
    pub edit: bool,
}

/// Draws one grid row (`rows` is that row's slice of streams); returns (stream index, action).
pub fn show_row(ui: &mut Ui, rows: &[StreamRow], cx: &RowCtx) -> Vec<(usize, RowAction)> {
    let mut out = Vec::new();
    let h = card_height(cx.card_w);
    let area = ui
        .allocate_exact_size(vec2(ui.available_width(), h), Sense::hover())
        .0;
    for (n, row) in rows.iter().enumerate() {
        let min = pos2(
            area.left() + n as f32 * (cx.card_w + theme::GAP),
            area.top(),
        );
        let card = Rect::from_min_size(min, vec2(cx.card_w, h));
        let idx = cx.first_index + n;
        for a in show_card(ui, row, card, idx == cx.cursor, cx) {
            out.push((idx, a));
        }
    }
    out
}

fn show_card(
    ui: &mut Ui,
    row: &StreamRow,
    rect: Rect,
    cursor: bool,
    cx: &RowCtx,
) -> Vec<RowAction> {
    let mut out = Vec::new();
    let mut ui = ui.new_child(UiBuilder::new().max_rect(rect));
    if row.hidden || row.missing {
        ui.set_opacity(0.5);
    }
    let base = ui.scope_id().with(("card", row.id));
    let resp = ui.interact(rect, base, Sense::click());
    let playing = cx.playing == Some(row.name.as_str());
    let border = if playing {
        theme::ACCENT
    } else if resp.hovered() || cursor {
        theme::CARD_BORDER_HOVER
    } else {
        theme::CARD_BORDER
    };
    let radius = CornerRadius::same(theme::RADIUS_CARD);
    ui.painter().rect_filled(rect, radius, theme::CARD);

    let poster = Rect::from_min_size(rect.min, vec2(rect.width(), rect.width() * 1.5));
    let top = CornerRadius {
        nw: theme::RADIUS_CARD,
        ne: theme::RADIUS_CARD,
        sw: 0,
        se: 0,
    };
    let drawn = row
        .logo
        .as_deref()
        .is_some_and(|u| tile::paint_image(&ui, u, poster, Fit::Cover, top));
    if !drawn {
        ui.painter().rect_filled(poster, top, theme::POSTER_BG);
        ui.painter().text(
            poster.center(),
            Align2::CENTER_CENTER,
            tile::initials(&row.name),
            theme::semibold(28.0),
            theme::TEXT_FAINT,
        );
    }
    let sub = row.resume_episode.as_ref().map(|e| e.label.as_str());
    paint_title(&ui, rect, poster.bottom(), &row.name, sub);
    progress_marks::paint_bar(&ui, poster, row.progress);
    if progress_marks::is_watched(row.progress) {
        progress_marks::paint_check(&ui, pos2(poster.left() + 16.0, poster.bottom() - 16.0));
    }
    ui.painter()
        .rect_stroke(rect, radius, Stroke::new(1.0, border), StrokeKind::Inside);

    let overlay = |x: f32| Rect::from_min_size(pos2(x, poster.top() + 6.0), vec2(30.0, 30.0));
    if row.favorite || resp.hovered() {
        let r = overlay(poster.right() - 36.0);
        paint_scrim(&ui, r);
        let (ic, tint, tip) = if row.favorite {
            (Icon::StarFilled, theme::STAR, "Remove favorite")
        } else {
            (Icon::Star, Color32::WHITE, "Add favorite")
        };
        if kit::icon_btn(&mut ui, r, base.with("fav"), ic, 16.0, tint, tip).clicked() {
            out.push(RowAction::ToggleFavorite);
        }
    }
    if cx.edit {
        let r = overlay(poster.left() + 6.0);
        paint_scrim(&ui, r);
        let (ic, tip) = if row.hidden {
            (Icon::EyeOff, "Show")
        } else {
            (Icon::Eye, "Hide")
        };
        if kit::icon_btn(&mut ui, r, base.with("eye"), ic, 16.0, Color32::WHITE, tip).clicked() {
            out.push(RowAction::ToggleHidden);
        }
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

fn paint_scrim(ui: &Ui, rect: Rect) {
    ui.painter()
        .rect_filled(rect, CornerRadius::same(8), Color32::from_black_alpha(140));
}

fn paint_title(ui: &Ui, card: Rect, top: f32, name: &str, sub: Option<&str>) {
    let mut job = LayoutJob::simple(
        name.to_owned(),
        theme::semibold(13.0),
        theme::TEXT,
        card.width() - 24.0,
    );
    job.wrap = TextWrapping {
        max_width: card.width() - 24.0,
        max_rows: if sub.is_some() { 1 } else { 2 },
        break_anywhere: false,
        overflow_character: Some('…'),
    };
    let galley = ui.painter().layout_job(job);
    ui.painter()
        .galley(pos2(card.left() + 12.0, top + 10.0), galley, theme::TEXT);
    if let Some(sub) = sub {
        let mut job = LayoutJob::simple(
            sub.to_owned(),
            theme::regular(11.5),
            theme::TEXT_MUTED,
            card.width() - 24.0,
        );
        job.wrap = TextWrapping {
            max_width: card.width() - 24.0,
            max_rows: 2,
            break_anywhere: false,
            overflow_character: Some('…'),
        };
        let galley = ui.painter().layout_job(job);
        ui.painter().galley(
            pos2(card.left() + 12.0, top + 30.0),
            galley,
            theme::TEXT_MUTED,
        );
    }
}
