//! Sidebar in edit mode: hint, drag-reorderable rows with hide/pin toggles, bottom toolbar.
use super::App;
use super::kit;
use super::theme::{self, Icon};
use crate::model::CategoryRow;
use egui::{CornerRadius, Frame, Margin, Rect, RichText, Sense, Ui, UiBuilder, pos2, vec2};
use egui_dnd::{DragDropItem, dnd};

const ROW_H: f32 = 40.0;

struct Item<'a>(&'a CategoryRow);

impl DragDropItem for Item<'_> {
    fn id(&self) -> egui::Id {
        egui::Id::unique(("cat", self.0.source_id, &self.0.key))
    }
}

enum Action {
    Hidden(usize, bool),
    Pinned(usize, bool),
}

impl App {
    pub(super) fn edit_ui(&mut self, ui: &mut Ui) {
        let single = self.single_source();
        let hint = match single {
            Some(s) => format!(
                "Drag rows to reorder. Order is saved for {}.",
                self.source_name(s)
            ),
            None => "Select one source to reorder categories.".to_string(),
        };
        Frame::NONE
            .fill(theme::INFO_BG)
            .corner_radius(theme::RADIUS)
            .inner_margin(Margin::symmetric(10, 8))
            .outer_margin(Margin {
                left: 12,
                right: 12,
                top: 0,
                bottom: 8,
            })
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(
                    RichText::new(hint)
                        .font(theme::regular(12.0))
                        .color(theme::INFO_FG),
                );
            });
        self.edit_toolbar(ui);

        let needle = self.edit.filter.to_lowercase();
        let mut actions = Vec::new();
        let mut finished = false;
        let tab = self.state.tab;
        let profile = self.profile();
        Frame::NONE
            .inner_margin(Margin {
                left: 10,
                right: 10,
                top: 0,
                bottom: 10,
            })
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink(false)
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing.y = 2.0;
                        let response = dnd(ui, egui::Id::unique("cat_dnd")).show(
                            self.categories.iter().map(Item),
                            |ui, item, handle, state| {
                                let c = item.0;
                                let label = self.cat_label(c);
                                let matched =
                                    !needle.is_empty() && c.name.to_lowercase().contains(&needle);
                                ui.scope(|ui| {
                                    if c.hidden {
                                        ui.set_opacity(0.45);
                                    }
                                    let rect = ui
                                        .allocate_exact_size(
                                            vec2(ui.available_width(), ROW_H),
                                            Sense::hover(),
                                        )
                                        .0;
                                    if matched {
                                        ui.painter().rect_filled(
                                            rect,
                                            CornerRadius::same(theme::RADIUS),
                                            theme::MATCH_BG,
                                        );
                                    } else if ui.rect_contains_pointer(rect) {
                                        ui.painter().rect_filled(
                                            rect,
                                            CornerRadius::same(theme::RADIUS),
                                            theme::ROW_HOVER,
                                        );
                                    }
                                    if single.is_some() {
                                        let grip = Rect::from_min_size(
                                            pos2(rect.left() + 4.0, rect.top()),
                                            vec2(22.0, ROW_H),
                                        );
                                        ui.scope_builder(UiBuilder::new().max_rect(grip), |ui| {
                                            handle.ui_sized(ui, grip.size(), |ui| {
                                                let at = Rect::from_center_size(
                                                    grip.center(),
                                                    vec2(14.0, 14.0),
                                                );
                                                theme::icon(Icon::Grip, 14.0, theme::TEXT_FAINT)
                                                    .show_loading_spinner(false)
                                                    .paint_at(ui, at);
                                            });
                                        });
                                    }
                                    let cy = rect.center().y;
                                    let button = |x: f32| {
                                        Rect::from_min_size(
                                            pos2(rect.left() + x, cy - 16.0),
                                            vec2(32.0, 32.0),
                                        )
                                    };
                                    let base = ui.scope_id().with(state.index);
                                    let (eye, tip) = if c.hidden {
                                        (Icon::EyeOff, "Show")
                                    } else {
                                        (Icon::Eye, "Hide")
                                    };
                                    if kit::icon_btn(
                                        ui,
                                        button(30.0),
                                        base.with("eye"),
                                        eye,
                                        16.0,
                                        theme::TEXT_MUTED,
                                        tip,
                                    )
                                    .clicked()
                                    {
                                        actions.push(Action::Hidden(state.index, !c.hidden));
                                    }
                                    let (star, tint, tip) = if c.pinned {
                                        (Icon::StarFilled, theme::STAR, "Unpin")
                                    } else {
                                        (Icon::Star, theme::TEXT_MUTED, "Pin to top")
                                    };
                                    if kit::icon_btn(
                                        ui,
                                        button(66.0),
                                        base.with("pin"),
                                        star,
                                        16.0,
                                        tint,
                                        tip,
                                    )
                                    .clicked()
                                    {
                                        actions.push(Action::Pinned(state.index, !c.pinned));
                                    }
                                    let left = kit::paint_badge(
                                        ui,
                                        rect.right() - 8.0,
                                        cy,
                                        &c.count.to_string(),
                                    );
                                    kit::paint_text(
                                        ui,
                                        rect.left() + 106.0,
                                        cy,
                                        left - rect.left() - 114.0,
                                        &label,
                                        theme::regular(13.0),
                                        theme::TEXT_ROW,
                                    );
                                });
                            },
                        );
                        finished = response.is_drag_finished();
                        if single.is_some() {
                            response.update_vec(&mut self.categories);
                        }
                    });
            });
        for a in actions {
            let (idx, hidden, pinned) = match a {
                Action::Hidden(i, h) => (i, Some(h), None),
                Action::Pinned(i, p) => (i, None, Some(p)),
            };
            let Some(c) = self.categories.get(idx) else {
                continue;
            };
            let (s, k) = (c.source_id, c.key.clone());
            let res = match (hidden, pinned) {
                (Some(h), _) => self.db.set_category_hidden(profile, s, tab, &k, h),
                (_, Some(p)) => self.db.set_category_pinned(profile, s, tab, &k, p),
                _ => Ok(()),
            };
            self.report(res);
            self.reload_categories();
        }
        if let (true, Some(source)) = (finished, single) {
            let keys: Vec<String> = self.categories.iter().map(|c| c.key.clone()).collect();
            let res = self.db.reorder_categories(profile, source, tab, &keys);
            self.report(res);
            self.reload_categories();
        }
    }
}
