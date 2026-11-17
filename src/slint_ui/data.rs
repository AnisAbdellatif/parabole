//! DB reloads, mirroring `ui/data.rs`.
use super::ToastItem;
use super::state::{Sel, SlintApp, with_app};
use crate::db::Db;
use crate::model::{MediaKind, SourceId};
use crate::smart::{self, Bucket, Variant};
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
        if self.toast_born.is_empty() {
            self.toast_timer.stop();
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

    /// Runs `job` on the writer thread; then reports its error and runs `then` on the UI thread.
    pub fn write(
        &self,
        job: impl FnOnce(&Db) -> anyhow::Result<()> + Send + 'static,
        then: impl FnOnce(&mut SlintApp) + Send + 'static,
    ) {
        self.write_with(job, move |a, _| then(a));
    }

    /// Like `write`, but `then` also learns whether the write succeeded.
    pub fn write_with(
        &self,
        job: impl FnOnce(&Db) -> anyhow::Result<()> + Send + 'static,
        then: impl FnOnce(&mut SlintApp, bool) + Send + 'static,
    ) {
        self.writer.run(move |db| {
            let res = job(db);
            let _ = slint::invoke_from_event_loop(move || {
                with_app(|a| {
                    let ok = res.is_ok();
                    a.report(res);
                    then(a, ok);
                });
            });
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
        self.push_profiles();
        self.reload_sources();
    }

    pub fn reload_sources(&mut self) {
        // Every profile switch and startup lands here.
        *self
            .worker
            .scan
            .profile
            .lock()
            .unwrap_or_else(|e| e.into_inner()) = Some(self.profile());
        match self.db.profile_sources(self.profile()) {
            Ok(s) => self.sources = s,
            Err(e) => self.toast(format!("{e:#}")),
        }
        if !self.sources.iter().any(|s| Some(s.id) == self.state.source) {
            self.state.source = None;
        }
        self.push_source_filter();
        self.reload_categories();
    }

    pub fn reload_categories(&mut self) {
        match self.db.categories(
            self.profile(),
            self.state.source,
            self.state.tab,
            self.browse.edit,
        ) {
            Ok(c) => self.categories = c,
            Err(e) => self.toast(format!("{e:#}")),
        }
        match self.db.groups(self.profile(), self.state.tab) {
            Ok(g) => self.groups = g,
            Err(e) => self.toast(format!("{e:#}")),
        }
        self.reload_smart();
        let valid = match &self.state.sel {
            Some(Sel::Quality(b)) => self.state.smart && Bucket::from_index(*b).is_some(),
            Some(_) if self.state.smart => false,
            Some(Sel::Group(id)) => self.groups.iter().any(|g| g.id == *id),
            Some(Sel::Category(s, k)) => self
                .categories
                .iter()
                .any(|c| c.source_id == *s && c.key == *k),
            Some(Sel::Continue) => self.state.tab != MediaKind::Live,
            Some(_) => true,
            None => false,
        };
        if !valid && self.state.smart {
            self.state.sel = Some(Sel::Quality(Bucket::All.index()));
        } else if !valid {
            self.state.sel = self
                .categories
                .iter()
                .find(|c| !c.hidden)
                .map(|c| Sel::Category(c.source_id, c.key.clone()));
        }
        self.push_sidebar();
        self.reload_streams();
    }

    fn reload_smart(&mut self) {
        if !self.state.smart {
            return self.smart.clear();
        }
        match self.db.live_variants(self.profile(), self.state.source) {
            Ok(v) => {
                let v = v.into_iter().map(|(row, quality)| Variant { row, quality });
                self.smart = smart::group(v.collect());
            }
            Err(e) => {
                self.smart.clear();
                self.toast(format!("{e:#}"));
            }
        }
    }

    pub fn reload_streams(&mut self) {
        let (profile, tab) = (self.profile(), self.state.tab);
        let sort = super::browse::load_sort(self);
        self.browse.sort = sort;
        let res = if !self.search_applied.is_empty() {
            self.db.search(
                profile,
                self.browse.filter.source(self.state.source),
                self.browse.filter.media(tab),
                &self.search_applied,
                SEARCH_LIMIT,
            )
        } else {
            match &self.state.sel {
                Some(Sel::Favorites) => self.db.favorites(profile, tab),
                Some(Sel::Recent) => self.db.recent(profile, tab, RECENT_LIMIT),
                Some(Sel::Continue) => self.db.continue_watching(profile, tab, RECENT_LIMIT),
                Some(Sel::Group(id)) => self.db.group_streams(profile, *id, tab),
                Some(Sel::Quality(b)) => Ok(self.smart_rows(*b)),
                Some(Sel::Category(s, k)) => self
                    .db
                    .streams_in_category(profile, *s, tab, k, self.browse.edit)
                    .map(|mut rows| {
                        sort.apply(&mut rows);
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

    pub fn source_name(&self, id: SourceId) -> &str {
        self.sources
            .iter()
            .find(|s| s.id == id)
            .map_or("?", |s| s.name.as_str())
    }
}
