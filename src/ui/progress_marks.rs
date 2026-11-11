//! Thin progress bar and "watched" check shared by cards, rows and episode rows.
use super::theme::{self, Icon};
use crate::db::progress::{Watch, classify};
use egui::{Color32, CornerRadius, Pos2, Rect, Ui, pos2, vec2};

const BAR_H: f32 = 3.0;
const CHECK: f32 = 18.0;

/// Accent bar along the bottom edge of `rect` for partially watched items.
pub fn paint_bar(ui: &Ui, rect: Rect, fraction: Option<f32>) {
    let Some(f) = fraction.filter(|_| classify(fraction) == Watch::Partial) else {
        return;
    };
    let track = Rect::from_min_max(
        pos2(rect.left(), rect.bottom() - BAR_H),
        rect.right_bottom(),
    );
    ui.painter()
        .rect_filled(track, CornerRadius::ZERO, Color32::from_black_alpha(150));
    let mut fill = track;
    fill.set_right(track.left() + track.width() * f.clamp(0.0, 1.0));
    ui.painter()
        .rect_filled(fill, CornerRadius::ZERO, theme::ACCENT);
}

pub fn is_watched(fraction: Option<f32>) -> bool {
    classify(fraction) == Watch::Watched
}

/// Round check badge centered at `center`.
pub fn paint_check(ui: &Ui, center: Pos2) {
    ui.painter()
        .circle_filled(center, CHECK / 2.0 + 2.0, Color32::from_black_alpha(170));
    theme::icon(Icon::Check, CHECK - 4.0, theme::ACCENT)
        .show_loading_spinner(false)
        .paint_at(
            ui,
            Rect::from_center_size(center, vec2(CHECK - 4.0, CHECK - 4.0)),
        );
}
