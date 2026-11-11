use super::App;
use super::state::{Sel, SeriesState};
use super::stream_row::{self, ROW_H, RowAction};
use crate::model::{MediaKind, SourceKind, StreamRow};
use crate::worker::Command;
use egui::Key;
use egui_dnd::{DragDropItem, dnd};

struct Fav<'a>(&'a StreamRow);

impl DragDropItem for Fav<'_> {
    fn id(&self) -> egui::Id {
        egui::Id::unique(("fav", self.0.id))
    }
}

const PAGE: usize = 15;
const MAX_DRAG_FAVORITES: usize = 300;

impl App {
    pub(super) fn center(&mut self, ui: &mut egui::Ui) {
        if self.sources.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(ui.available_height() / 3.0);
                    ui.heading("Add a source");
                    ui.label("Paste an M3U link, an Xtream login or pick a playlist file.");
                    if ui.button("Add a source").clicked() {
                        self.open_sources();
                    }
                });
            });
            return;
        }
        if self.series.is_some() {
            self.series_ui(ui);
            return;
        }
        ui.horizontal(|ui| {
            ui.heading(self.list_title());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                self.sort_selector(ui);
            });
        });
        let mut actions = self.key_actions(ui);
        let favorites = self.state.sel == Some(Sel::Favorites)
            && self.search.applied.is_empty()
            && self.streams.len() <= MAX_DRAG_FAVORITES;
        if self.streams.is_empty() {
            ui.weak("Nothing here.");
        } else if favorites {
            self.favorites_list(ui, &mut actions);
        } else {
            self.virtual_list(ui, &mut actions);
        }
        for (idx, action) in actions {
            self.row_action(idx, action);
        }
    }

    fn list_title(&self) -> String {
        if !self.search.applied.is_empty() {
            return format!("Search: “{}” ({})", self.search.applied, self.streams.len());
        }
        match &self.state.sel {
            Some(Sel::Favorites) => "★ Favorites".into(),
            Some(Sel::Recent) => "⟲ Recent".into(),
            Some(Sel::Category(s, k)) => self
                .categories
                .iter()
                .find(|c| c.source_id == *s && c.key == *k)
                .map_or(String::new(), |c| c.name.clone()),
            None => "Select a category".into(),
        }
    }

    fn key_actions(&mut self, ui: &egui::Ui) -> Vec<(usize, RowAction)> {
        let mut out = Vec::new();
        if ui.ctx().egui_wants_keyboard_input() || self.streams.is_empty() {
            return out;
        }
        let last = self.streams.len() - 1;
        let step = ui.input(|i| {
            let mut s = 0isize;
            s += i.key_pressed(Key::ArrowDown) as isize - i.key_pressed(Key::ArrowUp) as isize;
            s += PAGE as isize
                * (i.key_pressed(Key::PageDown) as isize - i.key_pressed(Key::PageUp) as isize);
            s
        });
        if step != 0 {
            self.selected = self.selected.saturating_add_signed(step).min(last);
            self.scroll_to_selected = true;
        }
        if ui.input(|i| i.key_pressed(Key::Enter)) {
            out.push((self.selected, RowAction::Activate));
        }
        out
    }

    fn virtual_list(&mut self, ui: &mut egui::Ui, actions: &mut Vec<(usize, RowAction)>) {
        let edit = self.edit.on;
        let spacing = ui.spacing().item_spacing.y;
        let mut area = egui::ScrollArea::vertical().auto_shrink(false);
        if std::mem::take(&mut self.scroll_to_selected) {
            let (offset, height) = self.list_view;
            let top = self.selected as f32 * (ROW_H + spacing);
            if top < offset {
                area = area.vertical_scroll_offset(top);
            } else if top + ROW_H > offset + height {
                area = area.vertical_scroll_offset(top + ROW_H - height);
            }
        }
        let out = area.show_rows(ui, ROW_H, self.streams.len(), |ui, range| {
            for i in range {
                for a in stream_row::show(ui, &self.streams[i], i == self.selected, edit) {
                    actions.push((i, a));
                }
            }
        });
        self.list_view = (out.state.clamped_offset().y, out.inner_rect.height());
    }

    // shortcut: favorites are not virtualized (drag reorder needs every row); above MAX_DRAG_FAVORITES the plain virtual list is used and reordering is off
    fn favorites_list(&mut self, ui: &mut egui::Ui, actions: &mut Vec<(usize, RowAction)>) {
        let mut finished = false;
        let edit = self.edit.on;
        egui::ScrollArea::vertical()
            .auto_shrink(false)
            .show(ui, |ui| {
                let response = dnd(ui, egui::Id::unique("fav_dnd")).show(
                    self.streams.iter().map(Fav),
                    |ui, item, handle, state| {
                        ui.horizontal(|ui| {
                            handle.ui(ui, |ui| {
                                ui.label("☰");
                            });
                            for a in
                                stream_row::show(ui, item.0, state.index == self.selected, edit)
                            {
                                actions.push((state.index, a));
                            }
                        });
                    },
                );
                finished = response.is_drag_finished();
                response.update_vec(&mut self.streams);
            });
        if finished {
            let items: Vec<_> = self
                .streams
                .iter()
                .map(|s| (s.source_id, s.key.clone()))
                .collect();
            let res = self
                .db
                .reorder_favorites(self.profile(), self.state.tab, &items);
            self.report(res);
        }
    }

    fn row_action(&mut self, idx: usize, action: RowAction) {
        let Some(row) = self.streams.get(idx).cloned() else {
            return;
        };
        let profile = self.profile();
        if row.missing && !matches!(action, RowAction::ToggleFavorite | RowAction::Select) {
            return;
        }
        match action {
            RowAction::Select => self.selected = idx,
            RowAction::Activate if self.state.tab == MediaKind::Series => self.open_series(&row),
            RowAction::Activate => self.play(&row, None),
            RowAction::Play(kind) => self.play(&row, Some(kind)),
            RowAction::Info => self.open_details(&row),
            RowAction::CopyUrl => self.worker.send(Command::CopyUrl { stream_id: row.id }),
            RowAction::ToggleFavorite => {
                let res = self.db.set_favorite(
                    profile,
                    row.source_id,
                    row.media,
                    &row.key,
                    !row.favorite,
                );
                self.report(res);
                self.reload_streams();
            }
            RowAction::ToggleHidden => {
                let res = self.db.set_stream_hidden(
                    profile,
                    row.source_id,
                    row.media,
                    &row.key,
                    !row.hidden,
                );
                self.report(res);
                self.reload_streams();
            }
        }
    }

    fn play(&mut self, row: &StreamRow, kind: Option<crate::player::PlayerKind>) {
        self.warn_if_connections_full(row.source_id);
        self.worker.send(Command::Play {
            profile: self.profile(),
            stream_id: row.id,
            kind,
        });
    }

    fn open_series(&mut self, row: &StreamRow) {
        let xtream = self
            .sources
            .iter()
            .any(|s| s.id == row.source_id && s.kind == SourceKind::Xtream);
        if !xtream {
            self.toast("Episode lists need an Xtream source");
            return;
        }
        self.series = Some(SeriesState {
            source: row.source_id,
            key: row.key.clone(),
            title: row.name.clone(),
            loading: true,
            seasons: Vec::new(),
            season: 0,
        });
        self.worker.send(Command::LoadSeries {
            source: row.source_id,
            key: row.key.clone(),
        });
    }
}
