use super::App;
use super::kit;
use super::state::Sel;
use super::theme;
use crate::model::CategoryRow;
use egui::{Align, Frame, Layout, Margin, RichText, Stroke, vec2};

impl App {
    pub(super) fn sidebar(&mut self, ui: &mut egui::Ui) {
        let panel = egui::Panel::left("sidebar")
            .resizable(false)
            .exact_size(theme::SIDEBAR_W)
            .frame(Frame::NONE.fill(theme::PANEL))
            .show(ui, |ui| {
                self.sidebar_header(ui);
                if self.edit.on {
                    self.edit_ui(ui);
                } else {
                    self.normal_ui(ui);
                }
            });
        let rect = panel.response.rect;
        ui.painter().vline(
            rect.right() - 0.5,
            rect.y_range(),
            Stroke::new(1.0, theme::DIVIDER),
        );
    }

    fn sidebar_header(&mut self, ui: &mut egui::Ui) {
        Frame::NONE
            .inner_margin(Margin {
                left: 18,
                right: 16,
                top: 14,
                bottom: 8,
            })
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("CATEGORIES")
                            .font(theme::semibold(12.0))
                            .color(theme::TEXT_MUTED),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let (label, on) = if self.edit.on {
                            ("Done", true)
                        } else {
                            ("Edit", false)
                        };
                        let size = vec2(kit::pill_width(ui, label) - 4.0, 30.0);
                        if kit::pill(ui, label, size, on, true).clicked() {
                            self.edit.on = !self.edit.on;
                            self.reload_categories();
                        }
                    });
                });
            });
    }

    pub(super) fn cat_label(&self, c: &CategoryRow) -> String {
        if self.show_source_prefix() {
            format!("{} · {}", self.source_name(c.source_id), c.name)
        } else {
            c.name.clone()
        }
    }

    pub(super) fn select(&mut self, sel: Sel) {
        self.state.sel = Some(sel);
        self.series = None;
        self.selected = 0;
        self.search.text.clear();
        self.search.applied.clear();
        self.search.filter = Default::default();
        self.reload_streams();
    }

    pub(super) fn report(&mut self, res: anyhow::Result<()>) {
        if let Err(e) = res {
            self.toast(format!("{e:#}"));
        }
    }
}
