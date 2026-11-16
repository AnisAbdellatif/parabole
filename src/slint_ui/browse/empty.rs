//! Messages for views with nothing to show (ports `ui/empty_state.rs`).
use super::groups::request_dialog;
use crate::model::{MediaKind, Source};
use crate::slint_ui::callbacks::clear_search;
use crate::slint_ui::push::MOD_LABEL;
use crate::slint_ui::state::{Sel, SlintApp, with_app};
use crate::slint_ui::{AppWindow, BrowseApi};
use crate::worker::Command;
use slint::ComponentHandle;

#[derive(Clone, Copy)]
pub enum Cta {
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

fn tab_name(tab: MediaKind) -> &'static str {
    match tab {
        MediaKind::Live => "Live",
        MediaKind::Movie => "Movies",
        MediaKind::Series => "Series",
    }
}

impl SlintApp {
    fn failed_sources(&self) -> impl Iterator<Item = &Source> {
        self.sources
            .iter()
            .filter(|s| s.last_error.is_some() && !self.refreshing.contains(&s.id))
    }

    fn empty_content(&self) -> Content {
        let tab = tab_name(self.state.tab);
        if self.sources.is_empty() {
            let body = "Paste an M3U link, an Xtream login or pick a playlist file.";
            return Content::new("Add a source", body).with("Add a source", Cta::ManageSources);
        }
        if !self.search_applied.is_empty() {
            let body = format!("Nothing in {tab} matches “{}”.", self.search_applied);
            return Content::new("No results", body).with("Clear search", Cta::ClearSearch);
        }
        match &self.state.sel {
            Some(Sel::Favorites) => {
                let body = format!("Star an item, or press {MOD_LABEL}D on the selected row.");
                return Content::new("No favorites yet", body);
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
            Some(Sel::Quality(_)) if !self.categories.is_empty() => {
                return Content::new("No channels here", "Pick another quality in the sidebar.");
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

    pub fn push_empty(&self) {
        let Some(win) = self.win.upgrade() else {
            return;
        };
        let c = self.empty_content();
        let api = win.global::<BrowseApi>();
        api.set_empty_title(c.title.into());
        api.set_empty_body(c.body.into());
        api.set_empty_cta(c.cta.map_or("", |(l, _)| l).into());
        api.set_empty_busy(c.busy);
        self.browse.cta.set(c.cta.map(|(_, a)| a));
    }
}

pub fn connect(win: &AppWindow) {
    win.global::<BrowseApi>().on_empty_cta_clicked(|| {
        with_app(|a| match a.browse.cta.get() {
            Some(Cta::ClearSearch) => {
                clear_search(a);
                a.reload_streams();
            }
            Some(Cta::Retry) => {
                for id in a.failed_sources().map(|s| s.id).collect::<Vec<_>>() {
                    a.worker.send(Command::Refresh(id));
                }
            }
            Some(Cta::ManageSources) => request_dialog(a, "sources"),
            None => {}
        });
    });
}
