//! Content-area chrome: title/subtitle header and the empty states.
use super::App;
use super::state::Sel;
use super::theme;
use crate::model::MediaKind;
use egui::{Align, Frame, Layout, Margin, RichText, Ui, vec2};

impl App {
    pub(super) fn list_header(&mut self, ui: &mut Ui) {
        let title = self.list_title();
        let n = self.streams.len();
        let subtitle = match n {
            0 => String::new(),
            1 => "1 item".to_string(),
            n => format!("{n} items"),
        };
        Frame::NONE
            .inner_margin(Margin {
                left: 24,
                right: 24,
                top: 18,
                bottom: 12,
            })
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(title).font(theme::semibold(20.0)));
                    ui.label(
                        RichText::new(subtitle)
                            .font(theme::regular(13.0))
                            .color(theme::TEXT_MUTED),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        self.sort_selector(ui);
                    });
                });
            });
    }

    fn list_title(&self) -> String {
        if !self.search.applied.is_empty() {
            return format!("Results for “{}”", self.search.applied);
        }
        match &self.state.sel {
            Some(Sel::Favorites) => "Favorites".into(),
            Some(Sel::Recent) => "Recent".into(),
            Some(Sel::Continue) => "Continue watching".into(),
            Some(Sel::Category(s, k)) => self
                .categories
                .iter()
                .find(|c| c.source_id == *s && c.key == *k)
                .map_or(String::new(), |c| c.name.clone()),
            None => "Select a category".into(),
        }
    }

    pub(super) fn empty_list(&self, ui: &mut Ui) {
        let text = if !self.search.applied.is_empty() {
            format!(
                "No matches in {}. Press Esc to clear.",
                tab_name(self.state.tab)
            )
        } else if self.state.sel == Some(Sel::Favorites) {
            "Nothing here yet. Star a channel to add it to Favorites.".to_string()
        } else if self.state.sel == Some(Sel::Continue) {
            "Nothing in progress. Started movies and episodes show up here.".to_string()
        } else {
            "Nothing here yet.".to_string()
        };
        ui.add_space(48.0);
        ui.vertical_centered(|ui| {
            ui.label(
                RichText::new(text)
                    .font(theme::regular(14.0))
                    .color(theme::TEXT_MUTED),
            );
        });
    }

    pub(super) fn no_sources(&mut self, ui: &mut Ui) {
        ui.add_space(ui.available_height() / 3.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("Add a source").font(theme::semibold(20.0)));
            ui.add_space(4.0);
            ui.label(
                RichText::new("Paste an M3U link, an Xtream login or pick a playlist file.")
                    .color(theme::TEXT_MUTED),
            );
            ui.add_space(12.0);
            let size = vec2(super::kit::pill_width(ui, "Add a source"), theme::CONTROL_H);
            if super::kit::pill(ui, "Add a source", size, true, true).clicked() {
                self.open_sources();
            }
        });
    }
}

fn tab_name(tab: MediaKind) -> &'static str {
    match tab {
        MediaKind::Live => "Live",
        MediaKind::Movie => "Movies",
        MediaKind::Series => "Series",
    }
}
