use super::App;
use super::poster_grid::{self, RowCtx};
use super::state::Sel;
use super::stream_row::{self, ROW_H, RowAction};
use super::theme::{self, Icon};
use crate::model::StreamRow;
use egui::{Frame, Key, Margin, Rect, vec2};
use egui_dnd::{DragDropItem, dnd};

struct Fav<'a>(&'a StreamRow);

impl DragDropItem for Fav<'_> {
    fn id(&self) -> egui::Id {
        egui::Id::unique(("fav", self.0.id))
    }
}

const PAGE: usize = 15;
const GRID_PAGE: usize = 3;
const MAX_DRAG_FAVORITES: usize = 300;
const LIST_GAP: f32 = 4.0;

type Actions = Vec<(usize, RowAction)>;

impl App {
    pub(super) fn center(&mut self, ui: &mut egui::Ui) {
        if self.sources.is_empty() {
            self.no_sources(ui);
            return;
        }
        if self.series.is_some() {
            self.series_ui(ui);
            return;
        }
        self.list_header(ui);
        self.publish_menu_data(ui.ctx());
        let favorites = matches!(self.state.sel, Some(Sel::Favorites | Sel::Group(_)))
            && self.search.applied.is_empty()
            && self.streams.len() <= MAX_DRAG_FAVORITES;
        let grid = self.grid_view() && !favorites;
        let mut actions = self.key_actions(ui, grid);
        Frame::NONE
            .inner_margin(Margin {
                left: 24,
                right: 24,
                top: 0,
                bottom: 24,
            })
            .show(ui, |ui| {
                if self.streams.is_empty() {
                    self.empty_list(ui);
                } else if favorites {
                    self.favorites_list(ui, &mut actions);
                } else {
                    self.virtual_list(ui, &mut actions, grid);
                }
            });
        if actions
            .iter()
            .any(|(_, a)| matches!(a, RowAction::LoadAlternates))
        {
            ui.ctx().request_repaint();
        }
        for (idx, action) in actions {
            self.row_action(idx, action);
        }
    }

    fn key_actions(&mut self, ui: &egui::Ui, grid: bool) -> Actions {
        let mut out = Vec::new();
        if ui.ctx().egui_wants_keyboard_input() || self.streams.is_empty() {
            return out;
        }
        let cols = if grid {
            poster_grid::columns(ui.available_width() - 48.0).0
        } else {
            1
        };
        let last = self.streams.len() - 1;
        let step = ui.input(|i| {
            let key = |k: Key| i.key_pressed(k) as isize;
            let rows = key(Key::ArrowDown) - key(Key::ArrowUp);
            let pages = key(Key::PageDown) - key(Key::PageUp);
            let page = if grid { GRID_PAGE } else { PAGE } as isize;
            let sideways = if grid {
                key(Key::ArrowRight) - key(Key::ArrowLeft)
            } else {
                0
            };
            (rows + pages * page) * cols as isize + sideways
        });
        if step != 0 {
            self.selected = self.selected.saturating_add_signed(step).min(last);
            self.scroll_to_selected = true;
        }
        let (home, end) = ui.input(|i| (i.key_pressed(Key::Home), i.key_pressed(Key::End)));
        if home || end {
            self.selected = if home { 0 } else { last };
            self.scroll_to_selected = true;
        }
        if ui.input(|i| i.key_pressed(Key::Enter)) {
            out.push((self.selected, RowAction::Activate));
        }
        out
    }

    fn virtual_list(&mut self, ui: &mut egui::Ui, actions: &mut Actions, grid: bool) {
        let edit = self.edit.on;
        let (cols, card_w) = if grid {
            poster_grid::columns(ui.available_width())
        } else {
            (1, 0.0)
        };
        let (row_h, gap) = if grid {
            (poster_grid::card_height(card_w), theme::GAP)
        } else {
            (ROW_H, LIST_GAP)
        };
        ui.spacing_mut().item_spacing.y = gap;
        let mut area = egui::ScrollArea::vertical().auto_shrink(false);
        if std::mem::take(&mut self.scroll_to_selected) {
            let (offset, height) = self.list_view;
            let top = (self.selected / cols) as f32 * (row_h + gap);
            if top < offset {
                area = area.vertical_scroll_offset(top);
            } else if top + row_h > offset + height {
                area = area.vertical_scroll_offset(top + row_h - height);
            }
        }
        let playing = self.playing.as_ref().map(|(title, _)| title.as_str());
        let total = self.streams.len().div_ceil(cols);
        let out = area.show_rows(ui, row_h, total, |ui, range| {
            for r in range {
                let start = r * cols;
                let end = (start + cols).min(self.streams.len());
                if grid {
                    let cx = RowCtx {
                        first_index: start,
                        card_w,
                        cursor: self.selected,
                        playing,
                        edit,
                    };
                    actions.extend(poster_grid::show_row(ui, &self.streams[start..end], &cx));
                } else {
                    let row = &self.streams[start];
                    let on = playing == Some(row.name.as_str());
                    for a in stream_row::show(ui, row, start == self.selected, on, edit) {
                        actions.push((start, a));
                    }
                }
            }
        });
        self.list_view = (out.state.clamped_offset().y, out.inner_rect.height());
    }

    // shortcut: favorites and group items are not virtualized (drag reorder needs every row) and use list rows on every tab; above MAX_DRAG_FAVORITES the virtual list/grid is used and reordering is off
    fn favorites_list(&mut self, ui: &mut egui::Ui, actions: &mut Actions) {
        let mut finished = false;
        let edit = self.edit.on;
        ui.spacing_mut().item_spacing.y = LIST_GAP;
        let playing = self.playing.as_ref().map(|(title, _)| title.as_str());
        egui::ScrollArea::vertical()
            .auto_shrink(false)
            .show(ui, |ui| {
                let response = dnd(ui, egui::Id::unique("fav_dnd")).show(
                    self.streams.iter().map(Fav),
                    |ui, item, handle, state| {
                        ui.horizontal(|ui| {
                            let size = vec2(22.0, ROW_H);
                            handle.ui_sized(ui, size, |ui| {
                                let at = Rect::from_center_size(
                                    ui.max_rect().center(),
                                    vec2(14.0, 14.0),
                                );
                                theme::icon(Icon::Grip, 14.0, theme::TEXT_FAINT)
                                    .show_loading_spinner(false)
                                    .paint_at(ui, at);
                            });
                            let on = playing == Some(item.0.name.as_str());
                            for a in
                                stream_row::show(ui, item.0, state.index == self.selected, on, edit)
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
            let res = match self.state.sel {
                Some(Sel::Group(g)) => self.db.reorder_group_items(g, &items),
                _ => self
                    .db
                    .reorder_favorites(self.profile(), self.state.tab, &items),
            };
            self.report(res);
        }
    }
}
