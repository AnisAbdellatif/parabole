//! Right-click menu shared by live rows and poster cards.
use super::menu_data::{self, MenuData};
use super::stream_row::RowAction;
use crate::model::{MediaKind, StreamRow};
use crate::player::PlayerKind;

pub fn show(ui: &mut egui::Ui, row: &StreamRow, out: &mut Vec<RowAction>) {
    let data = menu_data::current(ui);
    let in_group = data.as_ref().is_some_and(|d| d.in_group.is_some());
    if row.missing {
        if in_group {
            if ui.button("Remove from group").clicked() {
                out.push(RowAction::RemoveFromGroup);
                ui.close();
            }
        } else if ui.button("Remove from favorites").clicked() {
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
    if let Some(d) = data.as_deref().filter(|d| d.media == row.media) {
        group_items(ui, d, out);
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
    if let (MediaKind::Live, Some(d)) = (row.media, &data) {
        play_from(ui, d, row.id, out);
    }
    if ui.button("Copy stream URL").clicked() {
        out.push(RowAction::CopyUrl);
        ui.close();
    }
}

fn group_items(ui: &mut egui::Ui, d: &MenuData, out: &mut Vec<RowAction>) {
    ui.menu_button("Add to group ▸", |ui| {
        for g in &d.groups {
            if ui.button(&g.name).clicked() {
                out.push(RowAction::AddToGroup(g.id));
                ui.close();
            }
        }
        if !d.groups.is_empty() {
            ui.separator();
        }
        if ui.button("New group…").clicked() {
            out.push(RowAction::NewGroup);
            ui.close();
        }
    });
    if d.in_group.is_some() && ui.button("Remove from group").clicked() {
        out.push(RowAction::RemoveFromGroup);
        ui.close();
    }
}

fn play_from(ui: &mut egui::Ui, d: &MenuData, stream_id: i64, out: &mut Vec<RowAction>) {
    match &d.alts {
        Some((id, alts)) if *id == stream_id => {
            if alts.is_empty() {
                return;
            }
            ui.menu_button("Play from ▸", |ui| {
                for a in alts {
                    if ui.button(&a.source_name).clicked() {
                        out.push(RowAction::PlayFrom(a.stream_id));
                        ui.close();
                    }
                }
            });
        }
        _ => out.push(RowAction::LoadAlternates),
    }
}
