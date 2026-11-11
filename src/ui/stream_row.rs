use crate::model::{MediaKind, StreamRow};
use crate::player::PlayerKind;
use egui::{Align, Layout, Sense, UiBuilder, vec2};

pub const ROW_H: f32 = 28.0;
const LOGO: f32 = 22.0;

pub enum RowAction {
    Select,
    Activate,
    Play(PlayerKind),
    ToggleFavorite,
    ToggleHidden,
    CopyUrl,
    Info,
}

fn is_http(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://")
}

/// One list row; returns what the user asked for.
pub fn show(ui: &mut egui::Ui, row: &StreamRow, selected: bool, edit: bool) -> Vec<RowAction> {
    let mut out = Vec::new();
    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), ROW_H), Sense::click());
    if selected {
        ui.painter()
            .rect_filled(rect, 3.0, ui.visuals().selection.bg_fill);
    } else if resp.hovered() {
        ui.painter()
            .rect_filled(rect, 3.0, ui.visuals().widgets.hovered.weak_bg_fill);
    }
    let mut child = ui.new_child(
        UiBuilder::new()
            .max_rect(rect)
            .layout(Layout::left_to_right(Align::Center)),
    );
    child.style_mut().interaction.selectable_labels = false;
    let num = row.num.map(|n| n.to_string()).unwrap_or_default();
    child.add_sized(
        [40.0, ROW_H],
        egui::Label::new(egui::RichText::new(num).weak()),
    );
    if edit {
        let eye = if row.hidden { "🚫" } else { "👁" };
        if child
            .small_button(eye)
            .on_hover_text("Show / hide")
            .clicked()
        {
            out.push(RowAction::ToggleHidden);
        }
    }
    let star = if row.favorite { "★" } else { "☆" };
    if child.small_button(star).on_hover_text("Favorite").clicked() {
        out.push(RowAction::ToggleFavorite);
    }
    match row.logo.as_deref().filter(|u| is_http(u)) {
        Some(url) => {
            child.add(
                egui::Image::new(url)
                    .fit_to_exact_size(vec2(LOGO, LOGO))
                    .show_loading_spinner(false),
            );
        }
        None => child.add_space(LOGO),
    }
    let dim = row.hidden || row.missing;
    let name = match row.missing {
        true => format!("{} (unavailable)", row.name),
        false => row.name.clone(),
    };
    let name = egui::RichText::new(name);
    child.add(egui::Label::new(if dim { name.weak() } else { name }).truncate());

    if resp.clicked() {
        out.push(RowAction::Select);
    }
    if resp.double_clicked() && !row.missing {
        out.push(RowAction::Activate);
    }
    resp.context_menu(|ui| context_menu(ui, row, &mut out));
    out
}

fn context_menu(ui: &mut egui::Ui, row: &StreamRow, out: &mut Vec<RowAction>) {
    if row.missing {
        if ui.button("Remove from favorites").clicked() {
            out.push(RowAction::ToggleFavorite);
            ui.close();
        }
        return;
    }
    let fav = if row.favorite {
        "Unfavorite"
    } else {
        "Favorite"
    };
    if ui.button(fav).clicked() {
        out.push(RowAction::ToggleFavorite);
        ui.close();
    }
    if ui
        .button(if row.hidden { "Unhide" } else { "Hide" })
        .clicked()
    {
        out.push(RowAction::ToggleHidden);
        ui.close();
    }
    if row.media != MediaKind::Live && ui.button("Info…").clicked() {
        out.push(RowAction::Info);
        ui.close();
    }
    if row.media == MediaKind::Series {
        return;
    }
    ui.menu_button("Play with ▸", |ui| {
        for (label, kind) in [
            ("mpv", PlayerKind::Mpv),
            ("VLC", PlayerKind::Vlc),
            ("Custom", PlayerKind::Custom),
        ] {
            if ui.button(label).clicked() {
                out.push(RowAction::Play(kind));
                ui.close();
            }
        }
    });
    if ui.button("Copy stream URL").clicked() {
        out.push(RowAction::CopyUrl);
        ui.close();
    }
}
