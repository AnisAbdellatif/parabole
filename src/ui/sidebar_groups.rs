//! "My groups" rows of the normal sidebar.
use super::kit;
use super::sidebar_rows::{paint_row_bg, row_rect};
use super::theme;
use egui::{Id, Rect, Sense, Ui, pos2, vec2};

/// Returns true when "+" was clicked.
pub(super) fn groups_header(ui: &mut Ui) -> bool {
    let rect = row_rect(ui);
    kit::paint_text(
        ui,
        rect.left() + 10.0,
        rect.bottom() - 11.0,
        rect.width() - 50.0,
        "MY GROUPS",
        theme::semibold(11.0),
        theme::TEXT_DIM,
    );
    let plus = Rect::from_center_size(pos2(rect.right() - 20.0, rect.center().y), vec2(28.0, 28.0));
    let resp = ui
        .interact(plus, ui.scope_id().with("new_group"), Sense::click())
        .on_hover_text("New group");
    if resp.hovered() {
        ui.painter()
            .rect_filled(plus, theme::RADIUS, theme::ROW_HOVER);
    }
    kit::paint_text(
        ui,
        plus.center().x - 5.0,
        plus.center().y,
        14.0,
        "+",
        theme::semibold(16.0),
        theme::TEXT_MUTED,
    );
    resp.clicked()
}

pub(super) fn group_row(ui: &mut Ui, id: Id, label: &str, count: i64, selected: bool) -> bool {
    let rect = row_rect(ui);
    let row = ui.interact(rect, id, Sense::click());
    paint_row_bg(ui, rect, selected);
    let left = kit::paint_badge(ui, rect.right() - 10.0, rect.center().y, &count.to_string());
    let color = if selected {
        egui::Color32::WHITE
    } else {
        theme::TEXT_ROW
    };
    kit::paint_text(
        ui,
        rect.left() + 10.0,
        rect.center().y,
        left - rect.left() - 20.0,
        label,
        theme::regular(13.0),
        color,
    );
    row.clicked()
}
