use super::App;
use super::kit;
use super::progress_marks;
use super::theme::{self, Icon};
use crate::model::{MediaKind, Season, SourceId};
use crate::worker::{SeriesCtx, episode_key};
use egui::{CornerRadius, Frame, Margin, Rect, RichText, Sense, Ui, pos2, vec2};
use std::sync::Arc;
use std::time::Duration;

const EPISODE_H: f32 = 44.0;
const PROGRESS_EVERY: Duration = Duration::from_secs(3);

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

enum EpisodeEvent {
    Play,
    ToggleWatched,
}

/// One episode row with its progress bar and watched toggle.
fn episode_row(
    ui: &mut Ui,
    number: Option<i64>,
    title: &str,
    progress: Option<f32>,
) -> Option<EpisodeEvent> {
    let rect = ui
        .allocate_exact_size(vec2(ui.available_width(), EPISODE_H), Sense::hover())
        .0;
    let id = ui.scope_id().with(("ep", number, title));
    let resp = ui.interact(rect, id, Sense::click());
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
        rect.width() - 118.0,
        title,
        theme::regular(14.0),
        theme::TEXT,
    );
    progress_marks::paint_bar(ui, rect.shrink2(vec2(10.0, 0.0)), progress);
    let watched = progress_marks::is_watched(progress);
    let button = Rect::from_min_size(pos2(rect.right() - 38.0, cy - 16.0), vec2(32.0, 32.0));
    let mut event = resp.double_clicked().then_some(EpisodeEvent::Play);
    if watched || resp.hovered() {
        let (tint, tip) = if watched {
            (theme::ACCENT, "Mark unwatched")
        } else {
            (theme::TEXT_MUTED, "Mark watched")
        };
        if kit::icon_btn(ui, button, id.with("seen"), Icon::Check, 16.0, tint, tip).clicked() {
            event = Some(EpisodeEvent::ToggleWatched);
        }
    }
    event
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
            Ok(seasons) => {
                s.seasons = seasons;
                if let Some(id) = s.autoplay.take() {
                    self.play_series_episode(&id);
                }
            }
            Err(e) => {
                self.series = None;
                self.toast(format!("Could not load series: {e}"));
            }
        }
    }

    fn series_ctx(&self) -> Option<(SeriesCtx, SourceId)> {
        let s = self.series.as_ref()?;
        let ctx = SeriesCtx {
            key: s.key.clone(),
            title: s.title.clone(),
            seasons: Arc::new(s.seasons.clone()),
        };
        Some((ctx, s.source))
    }

    fn play_series_episode(&mut self, episode_id: &str) {
        let Some((ctx, source)) = self.series_ctx() else {
            return;
        };
        if let Some(cmd) = ctx.play_command(self.profile(), source, episode_id) {
            self.warn_if_connections_full(source);
            self.worker.send(cmd);
        }
    }

    fn toggle_watched(&mut self, episode_id: &str, watched: bool) {
        let Some((ctx, source)) = self.series_ctx() else {
            return;
        };
        let profile = self.profile();
        let key = episode_key(episode_id);
        let res = if watched {
            let label = crate::episodes::find(&ctx.seasons, episode_id)
                .map_or_else(String::new, |(s, e)| crate::episodes::label(s, e));
            self.db
                .record_episode_watch(profile, source, &key, &label, &ctx.key)
                .and_then(|_| {
                    self.db
                        .set_watched(profile, source, MediaKind::Series, &key, true)
                })
        } else {
            self.db
                .set_watched(profile, source, MediaKind::Series, &key, false)
        };
        self.report(res);
        if let Some(s) = self.series.as_mut() {
            s.progress_at = None;
        }
    }

    fn refresh_episode_progress(&mut self, ctx: &egui::Context) {
        let profile = self.profile();
        let Some(s) = self.series.as_mut() else {
            return;
        };
        if s.progress_at.is_some_and(|t| t.elapsed() < PROGRESS_EVERY) {
            ctx.request_repaint_after(PROGRESS_EVERY);
            return;
        }
        s.progress_at = Some(std::time::Instant::now());
        if let Ok(map) = self.db.episode_progress(profile, s.source) {
            s.progress = map;
        }
        ctx.request_repaint_after(PROGRESS_EVERY);
    }

    pub(super) fn series_ui(&mut self, ui: &mut egui::Ui) {
        self.refresh_episode_progress(&ui.ctx().clone());
        let Some(s) = self.series.as_mut() else {
            return;
        };
        let mut back = false;
        let mut play = None;
        let mut toggle = None;
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
                            let progress = s.progress.get(&episode_key(&ep.id)).copied();
                            match episode_row(ui, ep.episode_num, &ep.title, progress) {
                                Some(EpisodeEvent::Play) => play = Some(ep.id.clone()),
                                Some(EpisodeEvent::ToggleWatched) => {
                                    toggle =
                                        Some((ep.id.clone(), !progress_marks::is_watched(progress)))
                                }
                                None => {}
                            }
                        }
                    },
                );
            }
        });
        if let Some(id) = play {
            self.play_series_episode(&id);
        }
        if let Some((id, watched)) = toggle {
            self.toggle_watched(&id, watched);
        }
        if back {
            self.series = None;
        }
    }
}
