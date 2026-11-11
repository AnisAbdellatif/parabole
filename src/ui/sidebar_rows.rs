//! Normal (non-edit) sidebar body: Favorites/Recent entries and the virtualized category list.
use super::App;
use super::kit;
use super::state::Sel;
use super::theme::{self, Icon};
use crate::model::MediaKind;
use egui::{Frame, Id, Margin, Rect, Sense, Stroke, StrokeKind, Ui, pos2, vec2};

enum Line {
    Header(&'static str),
    Cat(usize),
}

const ROW_GAP: f32 = 2.0;

impl App {
    pub(super) fn normal_ui(&mut self, ui: &mut Ui) {
        let mut picked = None;
        let mut pin = None;
        Frame::NONE
            .inner_margin(Margin::symmetric(10, 0))
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = ROW_GAP;
                let mut entries = vec![
                    ("Favorites", Icon::StarFilled, Sel::Favorites),
                    ("Recent", Icon::Recent, Sel::Recent),
                ];
                if self.state.tab != MediaKind::Live {
                    entries.push(("Continue watching", Icon::Resume, Sel::Continue));
                }
                for (label, ic, sel) in entries {
                    let on = self.state.sel.as_ref() == Some(&sel);
                    if entry_row(ui, label, ic, on).clicked() {
                        picked = Some(sel);
                    }
                }
                ui.add_space(6.0);
                let (line, _) =
                    ui.allocate_exact_size(vec2(ui.available_width(), 1.0), Sense::hover());
                ui.painter()
                    .rect_filled(line.shrink2(vec2(6.0, 0.0)), 0.0, theme::DIVIDER);
                ui.add_space(8.0);

                let lines = self.lines();
                egui::ScrollArea::vertical().auto_shrink(false).show_rows(
                    ui,
                    theme::SIDEBAR_ROW_H,
                    lines.len(),
                    |ui, range| {
                        for (n, line) in lines[range.clone()].iter().enumerate() {
                            match line {
                                Line::Header(text) => header_row(ui, text),
                                Line::Cat(i) => {
                                    let c = &self.categories[*i];
                                    let sel = Sel::Category(c.source_id, c.key.clone());
                                    let on = self.state.sel.as_ref() == Some(&sel);
                                    let id = ui.scope_id().with(("cat", range.start + n));
                                    let (pick, pin_click) =
                                        cat_row(ui, id, &self.cat_label(c), c.count, on, c.pinned);
                                    if pick {
                                        picked = Some(sel);
                                    }
                                    if pin_click {
                                        pin = Some((c.source_id, c.key.clone(), !c.pinned));
                                    }
                                }
                            }
                        }
                    },
                );
            });
        if let Some((source, key, pinned)) = pin {
            let res =
                self.db
                    .set_category_pinned(self.profile(), source, self.state.tab, &key, pinned);
            self.report(res);
            self.reload_categories();
        }
        if let Some(sel) = picked {
            self.select(sel);
        }
    }

    fn lines(&self) -> Vec<Line> {
        let pinned = self.categories.iter().filter(|c| c.pinned).count();
        let mut out = Vec::with_capacity(self.categories.len() + 2);
        if pinned > 0 {
            out.push(Line::Header("PINNED"));
            out.extend(
                (0..self.categories.len())
                    .filter(|i| self.categories[*i].pinned)
                    .map(Line::Cat),
            );
        }
        if pinned < self.categories.len() {
            out.push(Line::Header(if pinned > 0 {
                "ALL CATEGORIES"
            } else {
                "CATEGORIES"
            }));
            out.extend(
                (0..self.categories.len())
                    .filter(|i| !self.categories[*i].pinned)
                    .map(Line::Cat),
            );
        }
        out
    }
}

fn row_rect(ui: &mut Ui) -> Rect {
    ui.allocate_exact_size(
        vec2(ui.available_width(), theme::SIDEBAR_ROW_H),
        Sense::hover(),
    )
    .0
}

fn paint_row_bg(ui: &Ui, rect: Rect, selected: bool) {
    let r = theme::RADIUS;
    if selected {
        ui.painter().rect_filled(rect, r, theme::SELECTED_BG);
        ui.painter().rect_stroke(
            rect,
            r,
            Stroke::new(1.0, theme::SELECTED_BORDER),
            StrokeKind::Inside,
        );
    } else if ui.rect_contains_pointer(rect) {
        ui.painter().rect_filled(rect, r, theme::ROW_HOVER);
    }
}

fn header_row(ui: &mut Ui, text: &str) {
    let rect = row_rect(ui);
    kit::paint_text(
        ui,
        rect.left() + 10.0,
        rect.bottom() - 11.0,
        rect.width() - 20.0,
        text,
        theme::semibold(11.0),
        theme::TEXT_DIM,
    );
}

fn entry_row(ui: &mut Ui, label: &str, ic: Icon, selected: bool) -> egui::Response {
    let rect = row_rect(ui);
    let resp = ui.interact(rect, ui.scope_id().with(label), Sense::click());
    paint_row_bg(ui, rect, selected);
    let color = if selected {
        egui::Color32::WHITE
    } else {
        theme::TEXT_ROW
    };
    let icon_rect =
        Rect::from_center_size(pos2(rect.left() + 18.0, rect.center().y), vec2(16.0, 16.0));
    theme::icon(ic, 16.0, color)
        .show_loading_spinner(false)
        .paint_at(ui, icon_rect);
    kit::paint_text(
        ui,
        rect.left() + 36.0,
        rect.center().y,
        rect.width() - 46.0,
        label,
        theme::regular(13.0),
        color,
    );
    resp
}

/// Returns (row clicked, pin button clicked).
fn cat_row(
    ui: &mut Ui,
    id: Id,
    label: &str,
    count: i64,
    selected: bool,
    pinned: bool,
) -> (bool, bool) {
    let rect = row_rect(ui);
    let row = ui.interact(rect, id, Sense::click());
    paint_row_bg(ui, rect, selected);
    let pin_rect = Rect::from_min_size(
        pos2(rect.right() - 36.0, rect.center().y - 16.0),
        vec2(32.0, 32.0),
    );
    let (ic, tint, tip) = if pinned {
        (Icon::StarFilled, theme::STAR, "Unpin")
    } else {
        (Icon::Star, theme::TEXT_MUTED, "Pin to top")
    };
    let pin = kit::icon_btn(ui, pin_rect, id.with("pin"), ic, 16.0, tint, tip);
    let left = kit::paint_badge(
        ui,
        pin_rect.left() - 4.0,
        rect.center().y,
        &count.to_string(),
    );
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
    (row.clicked(), pin.clicked())
}
