//! Small painted widgets shared by the sidebar and the content area.
use super::theme::{self, Icon};
use egui::text::{LayoutJob, TextWrapping};
use egui::{
    Align2, Color32, CornerRadius, FontId, Id, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui,
    Vec2, pos2, vec2,
};

/// Square icon button painted into `rect`; returns the click response.
pub fn icon_btn(
    ui: &mut Ui,
    rect: Rect,
    id: Id,
    ic: Icon,
    size: f32,
    color: Color32,
    tip: &str,
) -> Response {
    let resp = ui.interact(rect, id, Sense::click()).on_hover_text(tip);
    if resp.hovered() {
        ui.painter()
            .rect_filled(rect, theme::RADIUS, theme::SURFACE_HOVER);
    }
    let at = Rect::from_center_size(rect.center(), vec2(size, size));
    theme::icon(ic, size, color)
        .show_loading_spinner(false)
        .paint_at(ui, at);
    resp
}

/// Paints a count pill whose right edge is `right`; returns its left edge.
pub fn paint_badge(ui: &Ui, right: f32, cy: f32, text: &str) -> f32 {
    let galley =
        ui.painter()
            .layout_no_wrap(text.to_owned(), theme::semibold(11.0), theme::BADGE_TEXT);
    let size = galley.size();
    let pill = Rect::from_min_size(
        pos2(right - size.x - 16.0, cy - size.y / 2.0 - 2.0),
        vec2(size.x + 16.0, size.y + 4.0),
    );
    ui.painter()
        .rect_filled(pill, CornerRadius::same(255), theme::INPUT_BG);
    ui.painter().galley(
        pos2(pill.min.x + 8.0, cy - size.y / 2.0),
        galley,
        theme::BADGE_TEXT,
    );
    pill.min.x
}

/// Single-line text with its left edge at `left`, vertically centered on `cy`, cut with "…" at `max_w`.
pub fn paint_text(
    ui: &Ui,
    left: f32,
    cy: f32,
    max_w: f32,
    text: &str,
    font: FontId,
    color: Color32,
) {
    let mut job = LayoutJob::simple_singleline(text.to_owned(), font, color);
    job.wrap = TextWrapping::truncate_at_width(max_w.max(0.0));
    let galley = ui.painter().layout_job(job);
    let top_left: Pos2 = pos2(left, cy - galley.size().y / 2.0);
    ui.painter().galley(top_left, galley, color);
}

pub fn text_width(ui: &Ui, text: &str, font: FontId) -> f32 {
    ui.painter()
        .layout_no_wrap(text.to_owned(), font, theme::TEXT)
        .size()
        .x
}

/// Bordered button; `accent` is the filled primary variant.
pub fn pill(ui: &mut Ui, label: &str, size: Vec2, accent: bool, enabled: bool) -> Response {
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, resp) = ui.allocate_exact_size(size, sense);
    let (fill, border, fg) = if accent {
        (theme::ACCENT, theme::ACCENT, theme::ON_ACCENT)
    } else if resp.hovered() && enabled {
        (theme::SURFACE_HOVER, theme::BORDER, theme::TEXT)
    } else {
        (theme::SURFACE, theme::BORDER, theme::TEXT)
    };
    let fade = if enabled { 1.0 } else { 0.4 };
    let r = CornerRadius::same(theme::RADIUS);
    ui.painter().rect_filled(rect, r, fill.gamma_multiply(fade));
    ui.painter().rect_stroke(
        rect,
        r,
        Stroke::new(1.0, border.gamma_multiply(fade)),
        StrokeKind::Inside,
    );
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        label,
        theme::medium(13.0),
        fg.gamma_multiply(fade),
    );
    resp
}

/// Width a `pill` needs for `label` with 14px side padding.
pub fn pill_width(ui: &Ui, label: &str) -> f32 {
    text_width(ui, label, theme::medium(13.0)) + 28.0
}
