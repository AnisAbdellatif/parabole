//! Movie/series details: a window opened from the row menu, and a block reused atop the series view.
use super::App;
use super::theme;
use super::tile::is_http;
use crate::model::{MediaInfo, MediaKind, SourceId, SourceKind, StreamRow};
use crate::worker::Command;
use std::collections::{HashMap, HashSet};

type Key = (SourceId, MediaKind, String);

struct Open {
    key: Key,
    title: String,
    logo: Option<String>,
}

/// Per-session cache of fetched details plus the currently open window.
#[derive(Default)]
pub struct InfoStore {
    cache: HashMap<Key, MediaInfo>,
    loading: HashSet<Key>,
    failed: HashSet<Key>,
    open: Option<Open>,
}

impl InfoStore {
    pub fn info_for(&self, source: SourceId, media: MediaKind, key: &str) -> Option<&MediaInfo> {
        self.cache.get(&(source, media, key.to_string()))
    }
}

impl App {
    pub(super) fn open_details(&mut self, row: &StreamRow) {
        let key = (row.source_id, row.media, row.key.clone());
        self.info.open = Some(Open {
            key: key.clone(),
            title: row.name.clone(),
            logo: row.logo.clone(),
        });
        let xtream = self
            .sources
            .iter()
            .any(|s| s.id == row.source_id && s.kind == SourceKind::Xtream);
        if xtream && !self.info.cache.contains_key(&key) && self.info.loading.insert(key) {
            self.worker.send(Command::LoadInfo {
                source: row.source_id,
                media: row.media,
                key: row.key.clone(),
            });
        }
    }

    pub(super) fn info_loaded(
        &mut self,
        source: SourceId,
        media: MediaKind,
        key: String,
        result: Result<MediaInfo, String>,
    ) {
        let key = (source, media, key);
        self.info.loading.remove(&key);
        match result {
            Ok(info) => {
                self.info.cache.insert(key, info);
            }
            Err(e) => {
                // cache an empty entry so the window stops waiting and no retry loop starts
                self.info.failed.insert(key.clone());
                self.info.cache.insert(key, MediaInfo::default());
                self.toast(format!("Could not load details: {e}"));
            }
        }
    }

    pub(super) fn close_details(&mut self) -> bool {
        self.info.open.take().is_some()
    }

    pub(super) fn details_window(&mut self, ctx: &egui::Context) {
        let Some(open) = &self.info.open else {
            return;
        };
        let mut shown = true;
        let mut retry = false;
        egui::Window::new(&open.title)
            .open(&mut shown)
            .default_width(520.0)
            .show(ctx, |ui| match self.info.cache.get(&open.key) {
                Some(_) if self.info.failed.contains(&open.key) => {
                    let body = "The provider did not answer. Check the connection and try again.";
                    retry = super::empty_state::message(
                        ui,
                        "Could not load details",
                        body,
                        false,
                        Some("Retry"),
                    );
                }
                Some(info) => info_body(ui, info, open.logo.as_deref()),
                None if self.info.loading.contains(&open.key) => {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("Loading details…");
                    });
                }
                None => info_body(ui, &MediaInfo::default(), open.logo.as_deref()),
            });
        if retry {
            let (source, media, key) = open.key.clone();
            self.info.cache.remove(&open.key);
            self.info.failed.remove(&open.key);
            if self.info.loading.insert(open.key.clone()) {
                self.worker.send(Command::LoadInfo { source, media, key });
            }
        }
        if !shown {
            self.info.open = None;
        }
    }
}

/// Poster and fields; `fallback_poster` is used when the provider sent no cover.
pub fn info_body(ui: &mut egui::Ui, info: &MediaInfo, fallback_poster: Option<&str>) {
    let poster = info
        .cover
        .as_deref()
        .or(fallback_poster)
        .filter(|u| is_http(u));
    ui.horizontal_top(|ui| {
        if let Some(url) = poster {
            ui.add(
                egui::Image::new(url)
                    .max_width(150.0)
                    .corner_radius(theme::RADIUS_CARD),
            );
        }
        ui.vertical(|ui| {
            let fields = [
                ("Genre", &info.genre),
                ("Released", &info.release_date),
                ("Duration", &info.duration),
                ("Rating", &info.rating),
                ("Director", &info.director),
                ("Cast", &info.cast),
                ("Plot", &info.plot),
            ];
            let mut any = false;
            for (label, value) in fields {
                if let Some(v) = value {
                    any = true;
                    ui.horizontal_wrapped(|ui| {
                        ui.label(
                            egui::RichText::new(format!("{label}:"))
                                .font(theme::semibold(13.0))
                                .color(theme::TEXT_MUTED),
                        );
                        ui.label(v);
                    });
                }
            }
            if !any {
                ui.label(egui::RichText::new("No details available.").color(theme::TEXT_MUTED));
            }
        });
    });
}
