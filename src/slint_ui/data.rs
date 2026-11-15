//! DB reloads, mirroring `ui/data.rs` (no edit mode, alternates or search chips).
use super::ToastItem;
use super::state::{Sel, SlintApp};
use crate::model::{MediaKind, SourceId, StreamRow};
use slint::Model;
use std::time::Instant;

const RECENT_LIMIT: usize = 200;
const SEARCH_LIMIT: usize = 500;

impl SlintApp {
    pub fn toast(&mut self, text: impl Into<String>) {
        self.next_toast += 1;
        let id = self.next_toast;
        self.toasts.push(ToastItem {
            id,
            text: text.into().into(),
        });
        self.toast_born.push((id, Instant::now()));
        if !self.toast_timer.running() {
            self.toast_timer.start(
                slint::TimerMode::Repeated,
                std::time::Duration::from_millis(500),
                || {
                    super::state::with_app(|a| a.expire_toasts());
                },
            );
        }
    }

    pub fn remove_toast(&mut self, id: i32) {
        self.toast_born.retain(|(t, _)| *t != id);
        if let Some(i) = self.toasts.iter().position(|t| t.id == id) {
            self.toasts.remove(i);
        }
    }

    fn expire_toasts(&mut self) {
        let old: Vec<i32> = self
            .toast_born
            .iter()
            .filter(|(_, at)| at.elapsed().as_secs() >= 4)
            .map(|(id, _)| *id)
            .collect();
        for id in old {
            self.remove_toast(id);
        }
        if self.toast_born.is_empty() {
            self.toast_timer.stop();
        }
    }

    pub fn profile(&self) -> i64 {
        self.state.profile.unwrap_or_default()
    }

    pub fn report(&mut self, res: anyhow::Result<()>) {
        if let Err(e) = res {
            self.toast(format!("{e:#}"));
        }
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
        self.push_profiles();
        self.reload_sources();
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
        match self
            .db
            .categories(self.profile(), self.state.source, self.state.tab, false)
        {
            Ok(c) => self.categories = c,
            Err(e) => self.toast(format!("{e:#}")),
        }
        match self.db.groups(self.profile(), self.state.tab) {
            Ok(g) => self.groups = g,
            Err(e) => self.toast(format!("{e:#}")),
        }
        let valid = match &self.state.sel {
            Some(Sel::Group(id)) => self.groups.iter().any(|g| g.id == *id),
            Some(Sel::Category(s, k)) => self
                .categories
                .iter()
                .any(|c| c.source_id == *s && c.key == *k),
            Some(Sel::Continue) => self.state.tab != MediaKind::Live,
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
        self.push_sidebar();
        self.reload_streams();
    }

    pub fn reload_streams(&mut self) {
        let (profile, tab) = (self.profile(), self.state.tab);
        let res = if !self.search_applied.is_empty() {
            self.db.search(
                profile,
                self.state.source,
                Some(tab),
                &self.search_applied,
                SEARCH_LIMIT,
            )
        } else {
            match &self.state.sel {
                Some(Sel::Favorites) => self.db.favorites(profile, tab),
                Some(Sel::Recent) => self.db.recent(profile, tab, RECENT_LIMIT),
                Some(Sel::Continue) => self.db.continue_watching(profile, tab, RECENT_LIMIT),
                Some(Sel::Group(id)) => self.db.group_streams(profile, *id, tab),
                Some(Sel::Category(s, k)) => self
                    .db
                    .streams_in_category(profile, *s, tab, k, false)
                    .map(|mut rows| {
                        self.sort(&mut rows);
                        rows
                    }),
                None => Ok(Vec::new()),
            }
        };
        let rows = res.unwrap_or_else(|e| {
            self.toast(format!("{e:#}"));
            Vec::new()
        });
        self.push_streams(rows);
        self.push_status();
    }

    /// Per-category sort saved by the egui UI ("provider" | "number" | "name").
    fn sort(&self, rows: &mut [StreamRow]) {
        let Some(Sel::Category(s, k)) = &self.state.sel else {
            return;
        };
        let key = format!("sort:{s}:{}:{k}", self.state.tab.as_str());
        let mode = self
            .db
            .get_setting(Some(self.profile()), &key)
            .ok()
            .flatten();
        match mode.as_deref() {
            Some("number") => rows.sort_by_key(|r| (r.num.is_none(), r.num)),
            Some("name") => rows.sort_by_cached_key(|r| r.name.to_lowercase()),
            _ => {}
        }
    }

    pub fn source_name(&self, id: SourceId) -> &str {
        self.sources
            .iter()
            .find(|s| s.id == id)
            .map_or("?", |s| s.name.as_str())
    }
}
