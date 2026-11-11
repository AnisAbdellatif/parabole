use super::App;
use super::kit;
use super::theme::{self, Icon};
use crate::model::{MediaKind, Season, SourceId};
use crate::worker::Command;
use egui::{CornerRadius, Frame, Margin, Rect, RichText, Sense, Ui, pos2, vec2};

const EPISODE_H: f32 = 44.0;

fn back_pill(ui: &mut Ui, size: egui::Vec2) -> egui::Response {
    let resp = kit::pill(ui, "Back", size, false, true);
    let at = Rect::from_center_size(
        pos2(resp.rect.left() + 16.0, resp.rect.center().y),
        vec2(14.0, 14.0),
    );
    theme::icon(Icon::Back, 14.0, theme::TEXT)
        .show_loading_spinner(false)
        .paint_at(ui, at);
    resp
}

/// One episode row; true when double-clicked.
fn episode_row(ui: &mut Ui, number: Option<i64>, title: &str) -> bool {
    let rect = ui
        .allocate_exact_size(vec2(ui.available_width(), EPISODE_H), Sense::hover())
        .0;
    let resp = ui.interact(
        rect,
        ui.scope_id().with(("ep", number, title)),
        Sense::click(),
    );
    if resp.hovered() {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(theme::RADIUS_L), theme::ROW_HOVER);
    }
    let cy = rect.center().y;
    let num = number.map(|n| format!("E{n}")).unwrap_or_default();
    kit::paint_text(
        ui,
        rect.left() + 14.0,
        cy,
        44.0,
        &num,
        theme::semibold(12.0),
        theme::TEXT_MUTED,
    );
    kit::paint_text(
        ui,
        rect.left() + 68.0,
        cy,
        rect.width() - 82.0,
        title,
        theme::regular(14.0),
        theme::TEXT,
    );
    resp.double_clicked()
}

impl App {
    pub(super) fn series_loaded(
        &mut self,
        source: SourceId,
        key: &str,
        result: Result<Vec<Season>, String>,
    ) {
        let Some(s) = self
            .series
            .as_mut()
            .filter(|s| s.source == source && s.key == key)
        else {
            return;
        };
        s.loading = false;
        match result {
            Ok(seasons) => s.seasons = seasons,
            Err(e) => {
                self.series = None;
                self.toast(format!("Could not load series: {e}"));
            }
        }
    }

    pub(super) fn series_ui(&mut self, ui: &mut egui::Ui) {
        let Some(s) = self.series.as_mut() else {
            return;
        };
        let mut back = false;
        let mut play = None;
        let margin = Margin {
            left: 24,
            right: 24,
            top: 18,
            bottom: 24,
        };
        Frame::NONE.inner_margin(margin).show(ui, |ui| {
            ui.horizontal(|ui| {
                let size = vec2(kit::pill_width(ui, "Back") + 30.0, 30.0);
                back = back_pill(ui, size).clicked();
                ui.label(RichText::new(&s.title).font(theme::semibold(20.0)));
            });
            ui.add_space(12.0);
            if let Some(info) = self.info.info_for(s.source, MediaKind::Series, &s.key) {
                super::details_view::info_body(ui, info, None);
                ui.add_space(12.0);
            }
            if s.loading {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(RichText::new("Loading episodes…").color(theme::TEXT_MUTED));
                });
            } else if s.seasons.is_empty() {
                ui.label(RichText::new("No episodes found.").color(theme::TEXT_MUTED));
            } else {
                s.season = s.season.min(s.seasons.len() - 1);
                let name = |se: &Season| {
                    if se.name.is_empty() {
                        format!("Season {}", se.number)
                    } else {
                        se.name.clone()
                    }
                };
                egui::ComboBox::from_id_salt("season")
                    .selected_text(name(&s.seasons[s.season]))
                    .show_ui(ui, |ui| {
                        for (i, se) in s.seasons.iter().enumerate() {
                            ui.selectable_value(&mut s.season, i, name(se));
                        }
                    });
                ui.add_space(8.0);
                let season = &s.seasons[s.season];
                ui.spacing_mut().item_spacing.y = 4.0;
                egui::ScrollArea::vertical().auto_shrink(false).show_rows(
                    ui,
                    EPISODE_H,
                    season.episodes.len(),
                    |ui, range| {
                        for ep in &season.episodes[range] {
                            if episode_row(ui, ep.episode_num, &ep.title) {
                                let title = format!("{} S{} {}", s.title, season.number, ep.title);
                                play = Some((s.source, ep.clone(), title));
                            }
                        }
                    },
                );
            }
        });
        if let Some((source, episode, title)) = play {
            self.warn_if_connections_full(source);
            self.worker.send(Command::PlayEpisode {
                profile: self.profile(),
                source,
                episode,
                title,
                kind: None,
            });
        }
        if back {
            self.series = None;
        }
    }
}
