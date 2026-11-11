use super::*;
use egui::{CornerRadius, Stroke, TextStyle, Visuals};

pub fn install(ctx: &egui::Context) {
    ctx.set_theme(egui::Theme::Dark);
    ctx.style_mut_of(egui::Theme::Dark, |style| {
        style.visuals = visuals();
        style.text_styles = [
            (TextStyle::Small, regular(11.0)),
            (TextStyle::Body, regular(13.0)),
            (TextStyle::Button, medium(13.0)),
            (TextStyle::Heading, semibold(20.0)),
            (TextStyle::Monospace, egui::FontId::monospace(12.0)),
        ]
        .into();
        let s = &mut style.spacing;
        s.item_spacing = egui::vec2(8.0, 6.0);
        s.button_padding = egui::vec2(14.0, 8.0);
        s.interact_size.y = CONTROL_H;
        s.window_margin = egui::Margin::same(16);
        s.menu_margin = egui::Margin::same(6);
        s.combo_height = 320.0;
    });
}

fn visuals() -> Visuals {
    let mut v = Visuals::dark();
    v.override_text_color = Some(TEXT);
    v.panel_fill = PANEL;
    v.window_fill = PANEL;
    v.window_stroke = Stroke::new(1.0, BORDER);
    v.window_corner_radius = CornerRadius::same(RADIUS_CARD);
    v.menu_corner_radius = CornerRadius::same(RADIUS_L);
    v.extreme_bg_color = INPUT_BG;
    v.faint_bg_color = ROW_HOVER;
    v.code_bg_color = INPUT_BG;
    v.hyperlink_color = ACCENT;
    v.warn_fg_color = WARN;
    v.error_fg_color = ERROR;
    v.selection.bg_fill = SELECTED_BG;
    v.selection.stroke = Stroke::new(1.0, ACCENT);

    let r = CornerRadius::same(RADIUS);
    let w = &mut v.widgets;
    w.noninteractive.bg_fill = PANEL;
    w.noninteractive.weak_bg_fill = PANEL;
    w.noninteractive.bg_stroke = Stroke::new(1.0, DIVIDER);
    w.noninteractive.fg_stroke = Stroke::new(1.0, TEXT);
    w.noninteractive.corner_radius = r;
    for (state, fill) in [
        (&mut w.inactive, SURFACE),
        (&mut w.hovered, SURFACE_HOVER),
        (&mut w.active, SURFACE_HOVER),
        (&mut w.open, SURFACE_HOVER),
    ] {
        state.bg_fill = fill;
        state.weak_bg_fill = fill;
        state.bg_stroke = Stroke::new(1.0, BORDER);
        state.fg_stroke = Stroke::new(1.0, TEXT);
        state.corner_radius = r;
        state.expansion = 0.0;
    }
    w.hovered.fg_stroke = Stroke::new(1.5, egui::Color32::WHITE);
    w.active.bg_stroke = Stroke::new(1.0, ACCENT);
    v
}
