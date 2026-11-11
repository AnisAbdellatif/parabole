//! Remote images drawn into rects, plus the initials placeholders shown until they load.
use super::theme;
use egui::epaint::RectShape;
use egui::load::TexturePoll;
use egui::{Align2, Color32, CornerRadius, Rect, Stroke, StrokeKind, Ui, pos2, vec2};

pub fn is_http(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://")
}

pub fn initials(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|w| w.chars().find(|c| c.is_alphanumeric()))
        .take(2)
        .flat_map(char::to_uppercase)
        .collect()
}

#[derive(Clone, Copy, PartialEq)]
pub enum Fit {
    Contain,
    Cover,
}

/// Draws the image if it is already loaded; returns false (drawing nothing) otherwise.
pub fn paint_image(ui: &Ui, url: &str, rect: Rect, fit: Fit, corners: CornerRadius) -> bool {
    if !is_http(url) {
        return false;
    }
    let image = egui::Image::new(url);
    let Ok(TexturePoll::Ready { texture }) = image.load_for_size(ui.ctx(), rect.size()) else {
        return false;
    };
    if texture.size.x <= 0.0 || texture.size.y <= 0.0 {
        return false;
    }
    let ratio = rect.size() / texture.size;
    let (target, uv) = match fit {
        Fit::Contain => {
            let scale = ratio.x.min(ratio.y);
            let size = texture.size * scale;
            (
                Rect::from_center_size(rect.center(), size),
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            )
        }
        Fit::Cover => {
            let scale = ratio.x.max(ratio.y);
            let visible = rect.size() / (texture.size * scale);
            let margin = (vec2(1.0, 1.0) - visible) / 2.0;
            (
                rect,
                Rect::from_min_max(margin.to_pos2(), (vec2(1.0, 1.0) - margin).to_pos2()),
            )
        }
    };
    ui.painter()
        .add(RectShape::filled(target, corners, Color32::WHITE).with_texture(texture.id, uv));
    true
}

/// 44px rounded logo tile; initials show while there is no logo or it is still loading.
pub fn logo_tile(ui: &Ui, rect: Rect, name: &str, logo: Option<&str>) {
    let r = CornerRadius::same(theme::RADIUS_L);
    ui.painter().rect_filled(rect, r, theme::SURFACE);
    ui.painter()
        .rect_stroke(rect, r, Stroke::new(1.0, theme::BORDER), StrokeKind::Inside);
    let drawn = logo
        .is_some_and(|u| paint_image(ui, u, rect.shrink(5.0), Fit::Contain, CornerRadius::ZERO));
    if !drawn {
        ui.painter().text(
            rect.center(),
            Align2::CENTER_CENTER,
            initials(name),
            theme::semibold(13.0),
            theme::TEXT_MUTED,
        );
    }
}
