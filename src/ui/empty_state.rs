//! Centered messages for views with nothing to show, with an action button where one is obvious.
use super::App;
use super::state::Sel;
use super::theme;
use crate::model::MediaKind;
use crate::worker::Command;
use egui::{RichText, Ui, vec2};

#[derive(Clone, Copy)]
enum Cta {
    ClearSearch,
    Retry,
    ManageSources,
}

struct Content {
    title: String,
    body: String,
    cta: Option<(&'static str, Cta)>,
    busy: bool,
}

impl Content {
    fn new(title: &str, body: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            body: body.into(),
            cta: None,
            busy: false,
        }
    }

    fn with(mut self, label: &'static str, cta: Cta) -> Self {
        self.cta = Some((label, cta));
        self
    }
}

/// Title, body and optional button; returns true when the button was clicked.
pub(super) fn message(
    ui: &mut Ui,
    title: &str,
    body: &str,
    busy: bool,
    button: Option<&str>,
) -> bool {
    let mut clicked = false;
    ui.add_space((ui.available_height() / 4.0).clamp(24.0, 160.0));
    ui.vertical_centered(|ui| {
        if busy {
            ui.spinner();
            ui.add_space(8.0);
        }
        ui.label(RichText::new(title).font(theme::semibold(18.0)));
        if !body.is_empty() {
            ui.add_space(4.0);
            ui.label(
                RichText::new(body)
                    .font(theme::regular(14.0))
                    .color(theme::TEXT_MUTED),
            );
        }
        if let Some(label) = button {
            ui.add_space(12.0);
            let size = vec2(super::kit::pill_width(ui, label), theme::CONTROL_H);
            clicked = super::kit::pill(ui, label, size, true, true).clicked();
        }
    });
    clicked
}

fn tab_name(tab: MediaKind) -> &'static str {
    match tab {
        MediaKind::Live => "Live",
        MediaKind::Movie => "Movies",
        MediaKind::Series => "Series",
    }
}

impl App {
    fn failed_sources(&self) -> impl Iterator<Item = &crate::model::Source> {
        self.sources
            .iter()
            .filter(|s| s.last_error.is_some() && !self.refreshing.contains(&s.id))
    }

    fn empty_content(&self) -> Content {
        let tab = tab_name(self.state.tab);
        if !self.search.applied.is_empty() {
            let body = format!("Nothing in {tab} matches “{}”.", self.search.applied);
            return Content::new("No results", body).with("Clear search", Cta::ClearSearch);
        }
        match &self.state.sel {
            Some(Sel::Favorites) => {
                return Content::new(
                    "No favorites yet",
                    "Star an item, or press Ctrl+D on the selected row.",
                );
            }
            Some(Sel::Recent) => {
                return Content::new("Nothing watched yet", "What you play shows up here.");
            }
            Some(Sel::Continue) => {
                return Content::new(
                    "Nothing in progress",
                    "Started movies and episodes show up here.",
                );
            }
            Some(Sel::Group(_)) => {
                return Content::new("Empty group", "Right-click an item and pick Add to group.");
            }
            _ => {}
        }
        if self.categories.is_empty() {
            return self.no_categories(tab);
        }
        match self.state.sel {
            None => Content::new("Select a category", "Pick one from the sidebar."),
            Some(_) => Content::new("This category is empty", ""),
        }
    }

    fn no_categories(&self, tab: &str) -> Content {
        if !self.refreshing.is_empty() {
            let mut c = Content::new("Loading your sources…", "");
            c.busy = true;
            return c;
        }
        if let Some(err) = self.failed_sources().find_map(|s| s.last_error.as_deref()) {
            return Content::new("Refresh failed", err).with("Retry", Cta::Retry);
        }
        Content::new(
            &format!("No {tab} found"),
            "Your sources have nothing of this kind. Refresh them or add another.",
        )
        .with("Manage sources", Cta::ManageSources)
    }

    pub(super) fn empty_list(&mut self, ui: &mut Ui) {
        let c = self.empty_content();
        let label = c.cta.map(|(l, _)| l);
        if !message(ui, &c.title, &c.body, c.busy, label) {
            return;
        }
        match c.cta.map(|(_, a)| a) {
            Some(Cta::ClearSearch) => {
                self.clear_search();
            }
            Some(Cta::Retry) => {
                let ids: Vec<_> = self.failed_sources().map(|s| s.id).collect();
                for id in ids {
                    self.worker.send(Command::Refresh(id));
                }
            }
            Some(Cta::ManageSources) => self.open_sources(),
            None => {}
        }
    }

    pub(super) fn no_sources(&mut self, ui: &mut Ui) {
        let body = "Paste an M3U link, an Xtream login or pick a playlist file.";
        if message(ui, "Add a source", body, false, Some("Add a source")) {
            self.open_sources();
        }
    }
}
