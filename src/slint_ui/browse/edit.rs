//! Edit mode: categories and groups (ports `ui/edit_mode.rs`, `ui/edit_toolbar.rs`,
//! `ui/edit_groups.rs`, `ui/hide_new.rs`). Drag reorder is shortcut to move-up/down buttons.
use super::groups;
use crate::model::SourceId;
use crate::slint_ui::state::{SlintApp, with_app};
use crate::slint_ui::{AppWindow, BrowseApi, EditCat, EditGroup};
use slint::ComponentHandle;

impl SlintApp {
    /// Source whose category order can be edited: the filtered one, or the only one.
    pub fn single_source(&self) -> Option<SourceId> {
        self.state.source.or(match self.sources.as_slice() {
            [only] => Some(only.id),
            _ => None,
        })
    }

    pub fn push_edit(&self) {
        let Some(win) = self.win.upgrade() else {
            return;
        };
        let api = win.global::<BrowseApi>();
        api.set_edit_on(self.browse.edit);
        if !self.browse.edit {
            return;
        }
        let single = self.single_source();
        let hint = match single {
            Some(s) => format!(
                "Use ▲▼ to reorder rows. Order is saved for {}.",
                self.source_name(s)
            ),
            None => "Select one source to reorder categories.".to_string(),
        };
        api.set_edit_hint(hint.into());
        api.set_edit_can_reorder(single.is_some());
        let needle = self.browse.edit_filter.to_lowercase();
        api.set_edit_has_filter(!needle.is_empty());
        let cats = self.categories.iter().map(|c| EditCat {
            label: self.cat_label(c).into(),
            count: c.count as i32,
            hidden: c.hidden,
            pinned: c.pinned,
            matched: !needle.is_empty() && c.name.to_lowercase().contains(&needle),
        });
        self.browse.edit_cats.set_vec(cats.collect::<Vec<_>>());
        let groups = self.groups.iter().map(|g| EditGroup {
            name: g.name.clone().into(),
            confirm: self.browse.confirm_group == Some(g.id),
        });
        self.browse.edit_groups.set_vec(groups.collect::<Vec<_>>());
        let hide_new = single.and_then(|s| {
            self.db
                .hide_new_categories(self.profile(), s)
                .ok()
                .map(|on| (s, on))
        });
        api.set_hide_new_available(hide_new.is_some());
        api.set_hide_new(hide_new.is_some_and(|(_, on)| on));
    }

    fn bulk_hidden(&mut self, hide: impl Fn(&str) -> bool) {
        let changes: Vec<_> = self
            .categories
            .iter()
            .filter_map(|c| {
                let want = hide(&c.name);
                (want != c.hidden).then(|| (c.source_id, c.key.clone(), want))
            })
            .collect();
        let (profile, tab) = (self.profile(), self.state.tab);
        self.write(
            move |db| db.set_categories_hidden(profile, tab, &changes),
            |a| a.reload_categories(),
        );
    }

    fn save_category_order(&mut self, source: SourceId, keys: Vec<String>) {
        let (profile, tab) = (self.profile(), self.state.tab);
        self.write(
            move |db| db.reorder_categories(profile, source, tab, &keys),
            |a| a.reload_categories(),
        );
    }
}

fn set_edit(a: &mut SlintApp, on: bool) {
    a.browse.edit = on;
    a.browse.confirm_group = None;
    a.reload_categories();
}

/// Enters edit mode with the name filter pre-filled (sidebar "Hide matching…").
pub fn start_with_filter(a: &mut SlintApp, filter: String) {
    a.browse.edit_filter = filter.clone();
    if let Some(w) = a.win.upgrade() {
        w.global::<BrowseApi>().set_edit_filter(filter.into());
    }
    set_edit(a, true);
}

fn bulk(a: &mut SlintApp, what: &str) {
    let needle = a.browse.edit_filter.to_lowercase();
    let single = a.single_source();
    match (what, single) {
        ("show-all", _) => a.bulk_hidden(|_| false),
        ("hide-all", _) => a.bulk_hidden(|_| true),
        ("hide-match", _) if !needle.is_empty() => {
            a.bulk_hidden(|n| n.to_lowercase().contains(&needle));
        }
        ("show-match", _) if !needle.is_empty() => {
            a.bulk_hidden(|n| !n.to_lowercase().contains(&needle));
        }
        ("reset", Some(s)) => {
            let (profile, tab) = (a.profile(), a.state.tab);
            a.write(
                move |db| db.reset_category_order(profile, s, tab),
                |a| a.reload_categories(),
            );
        }
        ("sort-az", Some(s)) => {
            let mut sorted: Vec<_> = a.categories.iter().collect();
            sorted.sort_by_cached_key(|c| c.name.to_lowercase());
            let keys = sorted.iter().map(|c| c.key.clone()).collect();
            a.save_category_order(s, keys);
        }
        _ => {}
    }
}

fn move_item<T>(v: &mut [T], i: usize, delta: i32) -> bool {
    let j = i as i64 + delta as i64;
    if j < 0 || j as usize >= v.len() {
        return false;
    }
    v.swap(i, j as usize);
    true
}

pub fn connect(win: &AppWindow) {
    let api = win.global::<BrowseApi>();
    api.on_edit_toggled(|| {
        with_app(|a| {
            let on = !a.browse.edit;
            set_edit(a, on);
        });
    });
    api.on_edit_hide(|i| {
        with_app(|a| {
            let Some(c) = a.categories.get(i as usize) else {
                return;
            };
            let (s, k, h) = (c.source_id, c.key.clone(), !c.hidden);
            let (profile, tab) = (a.profile(), a.state.tab);
            a.write(
                move |db| db.set_category_hidden(profile, s, tab, &k, h),
                |a| a.reload_categories(),
            );
        });
    });
    api.on_edit_pin(|i| {
        with_app(|a| {
            let Some(c) = a.categories.get(i as usize) else {
                return;
            };
            let (s, k, p) = (c.source_id, c.key.clone(), !c.pinned);
            let (profile, tab) = (a.profile(), a.state.tab);
            a.write(
                move |db| db.set_category_pinned(profile, s, tab, &k, p),
                |a| a.reload_categories(),
            );
        });
    });
    api.on_edit_move(|i, d| {
        with_app(|a| {
            let Some(s) = a.single_source() else { return };
            if move_item(&mut a.categories, i as usize, d) {
                let keys = a.categories.iter().map(|c| c.key.clone()).collect();
                a.save_category_order(s, keys);
            }
        });
    });
    api.on_edit_bulk(|what| with_app(|a| bulk(a, &what)).unwrap_or_default());
    api.on_edit_filter_edited(|s| {
        with_app(|a| {
            a.browse.edit_filter = s.to_string();
            a.push_edit();
        });
    });
    api.on_hide_new_toggled(|on| {
        with_app(|a| {
            if let Some(s) = a.single_source() {
                let profile = a.profile();
                a.write(
                    move |db| db.set_hide_new_categories(profile, s, on),
                    |a| a.push_edit(),
                );
            }
        });
    });
    api.on_edit_group_move(|i, d| {
        with_app(|a| {
            if move_item(&mut a.groups, i as usize, d) {
                let ids: Vec<i64> = a.groups.iter().map(|g| g.id).collect();
                a.write(move |db| db.reorder_groups(&ids), |a| a.push_sidebar());
            }
        });
    });
    api.on_edit_group_rename(|i| {
        with_app(|a| {
            if let Some(g) = a.groups.get(i as usize) {
                let (id, name) = (g.id, g.name.clone());
                groups::open_group_dialog(a, Some(id), name, None);
            }
        });
    });
    api.on_edit_group_delete(|i| {
        with_app(|a| {
            let Some(id) = a.groups.get(i as usize).map(|g| g.id) else {
                return;
            };
            if a.browse.confirm_group == Some(id) {
                a.browse.confirm_group = None;
                groups::delete_group(a, id);
            } else {
                a.browse.confirm_group = Some(id);
                a.push_edit();
            }
        });
    });
}
