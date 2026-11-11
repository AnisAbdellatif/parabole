use super::App;
use super::state::{Sel, Toast};
use crate::model::SourceId;
use std::time::Instant;

const RECENT_LIMIT: usize = 200;
const SEARCH_LIMIT: usize = 500;

impl App {
    pub fn toast(&mut self, text: impl Into<String>) {
        self.toasts.push(Toast {
            text: text.into(),
            born: Instant::now(),
        });
    }

    pub fn reload_all(&mut self) {
        match self.db.profiles() {
            Ok(p) => self.profiles = p,
            Err(e) => self.toast(format!("{e:#}")),
        }
        if !self
            .profiles
            .iter()
            .any(|p| Some(p.id) == self.state.profile)
        {
            self.state.profile = self.profiles.first().map(|p| p.id);
        }
        self.reload_sources();
    }

    pub fn profile(&self) -> i64 {
        self.state.profile.unwrap_or_default()
    }

    pub fn reload_sources(&mut self) {
        match self.db.profile_sources(self.profile()) {
            Ok(s) => self.sources = s,
            Err(e) => self.toast(format!("{e:#}")),
        }
        if !self.sources.iter().any(|s| Some(s.id) == self.state.source) {
            self.state.source = None;
        }
        self.reload_categories();
    }

    pub fn reload_categories(&mut self) {
        let res = self.db.categories(
            self.profile(),
            self.state.source,
            self.state.tab,
            self.edit.on,
        );
        match res {
            Ok(c) => self.categories = c,
            Err(e) => self.toast(format!("{e:#}")),
        }
        let valid = match &self.state.sel {
            Some(Sel::Category(s, k)) => self
                .categories
                .iter()
                .any(|c| c.source_id == *s && c.key == *k),
            Some(_) => true,
            None => false,
        };
        if !valid {
            self.state.sel = self
                .categories
                .iter()
                .find(|c| !c.hidden)
                .map(|c| Sel::Category(c.source_id, c.key.clone()));
        }
        self.reload_streams();
    }

    pub fn reload_streams(&mut self) {
        let (profile, tab) = (self.profile(), self.state.tab);
        self.sort = self.load_sort();
        let sort = self.sort;
        let res = if !self.search.applied.is_empty() {
            self.db.search(
                profile,
                self.state.source,
                tab,
                &self.search.applied,
                SEARCH_LIMIT,
            )
        } else {
            match &self.state.sel {
                Some(Sel::Favorites) => self.db.favorites(profile, tab),
                Some(Sel::Recent) => self.db.recent(profile, tab, RECENT_LIMIT),
                Some(Sel::Category(s, k)) => self
                    .db
                    .streams_in_category(profile, *s, tab, k, self.edit.on)
                    .map(|mut rows| {
                        sort.apply(&mut rows);
                        rows
                    }),
                None => Ok(Vec::new()),
            }
        };
        match res {
            Ok(r) => self.streams = r,
            Err(e) => self.toast(format!("{e:#}")),
        }
        self.selected = self.selected.min(self.streams.len().saturating_sub(1));
    }

    pub fn source_name(&self, id: SourceId) -> &str {
        self.sources
            .iter()
            .find(|s| s.id == id)
            .map_or("?", |s| s.name.as_str())
    }

    pub fn show_source_prefix(&self) -> bool {
        self.state.source.is_none() && self.sources.len() > 1
    }

    /// Source whose category order can be edited: the filtered one, or the only one.
    pub fn single_source(&self) -> Option<SourceId> {
        self.state.source.or(match self.sources.as_slice() {
            [only] => Some(only.id),
            _ => None,
        })
    }

    pub fn set_tab(&mut self, tab: crate::model::MediaKind) {
        self.state.tab = tab;
        self.state.sel = None;
        self.series = None;
        self.selected = 0;
        self.reload_categories();
    }

    pub fn apply_search_if_due(&mut self, ctx: &egui::Context) {
        let Some(at) = self.search.edited_at else {
            return;
        };
        let wait = std::time::Duration::from_millis(150);
        if at.elapsed() < wait {
            ctx.request_repaint_after(wait);
            return;
        }
        self.search.edited_at = None;
        self.search.applied = self.search.text.trim().to_string();
        self.series = None;
        self.selected = 0;
        self.reload_streams();
    }
}
