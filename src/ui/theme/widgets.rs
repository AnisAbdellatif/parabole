//! Generic buttons shared by dialogs and bars.
use super::*;
use egui::{Button, Response, RichText, Stroke, Ui};

/// Accent-filled call-to-action button.
pub fn primary_button(ui: &mut Ui, text: &str) -> Response {
    ui.add(
        Button::new(RichText::new(text).font(semibold(13.0)).color(ON_ACCENT))
            .fill(ACCENT)
            .stroke(Stroke::NONE),
    )
}

pub fn secondary_button(ui: &mut Ui, text: &str) -> Response {
    ui.add(Button::new(RichText::new(text).font(medium(13.0))).fill(SURFACE))
}

/// Square-ish button with a tintable icon and a hover tooltip.
pub fn icon_button(ui: &mut Ui, i: Icon, tooltip: &str) -> Response {
    ui.add(Button::image(icon(i, 15.0, TEXT)).fill(SURFACE))
        .on_hover_text(tooltip)
}
