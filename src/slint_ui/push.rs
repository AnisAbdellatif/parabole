//! Copies app state into the `Core` global's properties.
use super::state::{Sel, SlintApp};
use super::{Core, SidebarItem};
use crate::model::{MediaKind, StreamRow};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};

fn item(id: String, kind: i32, label: String, count: i32, icon: i32) -> SidebarItem {
    SidebarItem {
        id: id.into(),
        kind,
        label: label.into(),
        count,
        icon,
    }
}

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

    pub fn push_sidebar(&self) {
        let Some(win) = self.win.upgrade() else {
            return;
        };
        let prefix = self.state.source.is_none() && self.sources.len() > 1;
        let mut v = vec![item(String::new(), 0, "LIBRARY".into(), -1, 0)];
        v.push(item(Sel::Favorites.to_id(), 1, "Favorites".into(), -1, 1));
        v.push(item(Sel::Recent.to_id(), 1, "Recent".into(), -1, 2));
        if self.state.tab != MediaKind::Live {
            v.push(item(
                Sel::Continue.to_id(),
                1,
                "Continue watching".into(),
                -1,
                3,
            ));
        }
        if !self.groups.is_empty() {
            v.push(item(String::new(), 0, "GROUPS".into(), -1, 0));
            for g in &self.groups {
                v.push(item(
                    Sel::Group(g.id).to_id(),
                    2,
                    g.name.clone(),
                    g.count as i32,
                    0,
                ));
            }
        }
        v.push(item(String::new(), 0, "CATEGORIES".into(), -1, 0));
        for c in self.categories.iter().filter(|c| !c.hidden) {
            let label = if prefix {
                format!("{} · {}", self.source_name(c.source_id), c.name)
            } else {
                c.name.clone()
            };
            let id = Sel::Category(c.source_id, c.key.clone()).to_id();
            v.push(item(id, 3, label, c.count as i32, 0));
        }
        let core = win.global::<Core>();
        core.set_sidebar(ModelRc::new(VecModel::from(v)));
        core.set_selected_id(
            self.state
                .sel
                .as_ref()
                .map(Sel::to_id)
                .unwrap_or_default()
                .into(),
        );
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
    }
}
