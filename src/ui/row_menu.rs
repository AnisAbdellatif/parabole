//! Right-click menu shared by live rows and poster cards.
use super::stream_row::RowAction;
use crate::model::{MediaKind, StreamRow};
use crate::player::PlayerKind;

pub fn show(ui: &mut egui::Ui, row: &StreamRow, out: &mut Vec<RowAction>) {
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
