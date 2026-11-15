//! Copies app state into the `Core` global's properties.
use super::Core;
use super::state::{Sel, SlintApp};
use crate::model::StreamRow;
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};

impl SlintApp {
    pub fn push_profiles(&self) {
        let Some(win) = self.win.upgrade() else {
            return;
        };
        let core = win.global::<Core>();
        let names: Vec<SharedString> = self
            .profiles
            .iter()
            .map(|p| p.name.as_str().into())
            .collect();
        let idx = self
            .profiles
            .iter()
            .position(|p| Some(p.id) == self.state.profile)
            .unwrap_or(0);
        core.set_profile_name(names.get(idx).cloned().unwrap_or_default());
        core.set_profile_index(idx as i32);
        core.set_profiles(ModelRc::new(VecModel::from(names)));
    }

    pub fn push_streams(&mut self, rows: Vec<StreamRow>) {
        let n = rows.len();
        let prefix = self.state.source.is_none() && self.sources.len() > 1;
        let names = self
            .sources
            .iter()
            .map(|s| (s.id, s.name.clone()))
            .collect();
        self.streams.set_rows(rows, prefix, names);
        let Some(win) = self.win.upgrade() else {
            return;
        };
        let core = win.global::<Core>();
        let title = if !self.search_applied.is_empty() {
            format!("Search: {}", self.search_applied)
        } else {
            match &self.state.sel {
                Some(Sel::Favorites) => "Favorites".into(),
                Some(Sel::Recent) => "Recent".into(),
                Some(Sel::Continue) => "Continue watching".into(),
                Some(Sel::Group(id)) => self
                    .groups
                    .iter()
                    .find(|g| g.id == *id)
                    .map(|g| g.name.clone())
                    .unwrap_or_default(),
                Some(Sel::Category(s, k)) => self
                    .categories
                    .iter()
                    .find(|c| c.source_id == *s && c.key == *k)
                    .map(|c| c.name.clone())
                    .unwrap_or_default(),
                None => String::new(),
            }
        };
        core.set_list_title(title.into());
        core.set_list_subtitle(format!("{n} items").into());
        let empty = if self.sources.is_empty() {
            "Add a source to get started"
        } else if !self.search_applied.is_empty() {
            "No matches"
        } else {
            "Nothing here yet"
        };
        core.set_empty_text(empty.into());
        core.set_cursor(core.get_cursor().min(n.saturating_sub(1) as i32));
        self.push_browse_header();
        self.push_empty();
    }

    pub fn push_status(&self) {
        let Some(win) = self.win.upgrade() else {
            return;
        };
        let core = win.global::<Core>();
        let n = self.sources.len();
        core.set_has_sources(n > 0);
        core.set_syncing(!self.refreshing.is_empty());
        let text = if self.refreshing.is_empty() {
            format!("{n} source{} synced", if n == 1 { "" } else { "s" })
        } else {
            "Syncing…".into()
        };
        core.set_status_sources(text.into());
        let items: i64 = self.categories.iter().map(|c| c.count).sum();
        core.set_status_items(
            format!("{items} items in {} categories", self.categories.len()).into(),
        );
        self.push_empty();
    }
}
