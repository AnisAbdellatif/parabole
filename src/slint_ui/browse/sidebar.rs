//! Sidebar rows, group/category menu, pinning, keyboard focus
//! (ports `ui/sidebar_rows.rs`, `ui/sidebar_groups.rs`, the sidebar half of `ui/nav_keys.rs`).
use super::groups;
use super::sort::SortMode;
use crate::model::MediaKind;
use crate::slint_ui::callbacks::select;
use crate::slint_ui::state::{Sel, SlintApp, with_app};
use crate::slint_ui::{AppWindow, BrowseApi, Core, MenuEntry, SideRow};
use slint::{ComponentHandle, Model, ModelRc, VecModel};

fn row(id: String, kind: i32, label: String, count: i32, icon: i32, pinned: bool) -> SideRow {
    SideRow {
        id: id.into(),
        kind,
        label: label.into(),
        count,
        icon,
        pinned,
    }
}

fn header(label: &str) -> SideRow {
    row(String::new(), 0, label.into(), -1, 0, false)
}

/// Case-insensitive substring match; an empty filter keeps everything.
fn matches(label: &str, filter: &str) -> bool {
    filter.is_empty() || label.to_lowercase().contains(&filter.to_lowercase())
}

impl SlintApp {
    pub fn push_sidebar(&self) {
        if self.state.smart {
            return self.push_smart_sidebar();
        }
        let Some(win) = self.win.upgrade() else {
            return;
        };
        let filter = win.global::<BrowseApi>().get_side_filter();
        let mut v = vec![
            row(Sel::Favorites.to_id(), 1, "Favorites".into(), -1, 1, false),
            row(Sel::Recent.to_id(), 1, "Recent".into(), -1, 2, false),
        ];
        if self.state.tab != MediaKind::Live {
            let id = Sel::Continue.to_id();
            v.push(row(id, 1, "Continue watching".into(), -1, 3, false));
        }
        let groups: Vec<_> = self
            .groups
            .iter()
            .filter(|g| matches(&g.name, &filter))
            .collect();
        if filter.is_empty() || !groups.is_empty() {
            v.push(row(String::new(), 4, "MY GROUPS".into(), -1, 0, false));
        }
        for g in &groups {
            let id = Sel::Group(g.id).to_id();
            v.push(row(id, 2, g.name.clone(), g.count as i32, 0, false));
        }
        let cats: Vec<_> = self
            .categories
            .iter()
            .filter(|c| !c.hidden)
            .map(|c| (c, self.cat_label(c)))
            .filter(|(_, label)| matches(label, &filter))
            .collect();
        let pinned = cats.iter().filter(|(c, _)| c.pinned).count();
        for (title, want) in [("PINNED", true), ("", false)] {
            let mut part = cats.iter().filter(|(c, _)| c.pinned == want).peekable();
            if part.peek().is_none() {
                continue;
            }
            let title = match (want, pinned) {
                (false, 0) => "CATEGORIES",
                (false, _) => "ALL CATEGORIES",
                _ => title,
            };
            v.push(header(title));
            for (c, label) in part {
                let id = Sel::Category(c.source_id, c.key.clone()).to_id();
                v.push(row(id, 3, label.clone(), c.count as i32, 0, c.pinned));
            }
        }
        if groups.is_empty() && cats.is_empty() && !filter.is_empty() {
            v.push(header("No categories match"));
        }
        let sel_id = self.state.sel.as_ref().map(Sel::to_id).unwrap_or_default();
        let at = v
            .iter()
            .position(|r| !sel_id.is_empty() && r.id == sel_id.as_str());
        self.browse.side_rows.set_vec(v);
        let core = win.global::<Core>();
        core.set_selected_index(at.map_or(-1, |i| i as i32));
        core.set_selected_id(sel_id.into());
        self.push_edit();
    }

    pub fn show_source_prefix(&self) -> bool {
        self.state.source.is_none() && self.sources.len() > 1
    }

    pub fn cat_label(&self, c: &crate::model::CategoryRow) -> String {
        if self.show_source_prefix() {
            format!("{} · {}", self.source_name(c.source_id), c.name)
        } else {
            c.name.clone()
        }
    }

    /// Selectable sidebar entries top to bottom, as currently shown (filter applied).
    fn sidebar_order(&self) -> Vec<Sel> {
        self.browse
            .side_rows
            .iter()
            .filter_map(|r| Sel::from_id(&r.id))
            .collect()
    }

    fn scroll_sidebar_to_selection(&self) {
        let Some(win) = self.win.upgrade() else {
            return;
        };
        let id = self.state.sel.as_ref().map(Sel::to_id).unwrap_or_default();
        let at = self.browse.side_rows.iter().position(|r| r.id == id);
        let api = win.global::<BrowseApi>();
        if let Some(at) = at {
            api.set_side_scroll_index(at as i32);
            api.set_side_scroll_tick(api.get_side_scroll_tick() + 1);
        }
    }
}

fn entry(id: &str, label: String, kind: i32) -> MenuEntry {
    MenuEntry {
        id: id.into(),
        label: label.into(),
        kind,
    }
}

fn sort_key(tab: MediaKind, s: i64, k: &str) -> String {
    format!("sort:{s}:{}:{k}", tab.as_str())
}

fn menu_entries(a: &SlintApp, sel: &Sel) -> Vec<MenuEntry> {
    match sel {
        Sel::Category(s, k) => {
            let pinned = a
                .categories
                .iter()
                .any(|c| c.source_id == *s && c.key == *k && c.pinned);
            let cur =
                a.db.get_setting(Some(a.profile()), &sort_key(a.state.tab, *s, k))
                    .ok()
                    .flatten()
                    .map_or(SortMode::Provider, |v| SortMode::parse(&v));
            let mut v = vec![
                entry("hide", "Hide category".into(), 0),
                entry("pin", if pinned { "Unpin" } else { "Pin to top" }.into(), 0),
                entry("", String::new(), 2),
                entry("", "SORT".into(), 1),
            ];
            for m in SortMode::ALL {
                let mark = if m == cur { "✓ " } else { "    " };
                v.push(entry(
                    &format!("sort:{}", m.as_str()),
                    format!("{mark}{}", m.label()),
                    0,
                ));
            }
            v.push(entry("", String::new(), 2));
            v.push(entry("hidematch", "Hide matching…".into(), 0));
            v
        }
        Sel::Group(_) => vec![
            entry("rename", "Rename…".into(), 0),
            entry("editgroups", "Edit groups…".into(), 0),
        ],
        _ => Vec::new(),
    }
}

/// "FR | Cinema" -> "FR |": the prefix providers use to namespace categories.
fn prefix_of(name: &str) -> String {
    match name.find(['|', ':', '-']) {
        Some(i) => name[..=i].trim().to_owned(),
        None => name.split_whitespace().next().unwrap_or("").to_owned(),
    }
}

fn menu_picked(a: &mut SlintApp, sel: Sel, id: &str) {
    let (profile, tab) = (a.profile(), a.state.tab);
    match (&sel, id) {
        (Sel::Category(s, k), "hide") => {
            let (s, k) = (*s, k.clone());
            a.write(
                move |db| db.set_category_hidden(profile, s, tab, &k, true),
                |a| a.reload_categories(),
            );
        }
        (Sel::Category(s, k), "pin") => toggle_pin(a, *s, k),
        (Sel::Category(s, k), "hidematch") => {
            let name = a
                .categories
                .iter()
                .find(|c| c.source_id == *s && c.key == *k)
                .map(|c| prefix_of(&c.name))
                .unwrap_or_default();
            super::edit::start_with_filter(a, name);
        }
        (Sel::Category(s, k), _) if id.starts_with("sort:") => {
            let mode = SortMode::parse(&id["sort:".len()..]);
            let key = sort_key(tab, *s, k);
            a.write(
                move |db| db.set_setting(Some(profile), &key, mode.as_str()),
                |a| a.reload_streams(),
            );
        }
        (Sel::Group(g), "rename") => {
            let name = a
                .groups
                .iter()
                .find(|x| x.id == *g)
                .map(|x| x.name.clone())
                .unwrap_or_default();
            groups::open_group_dialog(a, Some(*g), name, None);
        }
        (Sel::Group(_), "editgroups") => super::edit::start_with_filter(a, String::new()),
        _ => {}
    }
}

fn toggle_pin(a: &mut SlintApp, s: i64, k: &str) {
    let pinned = a
        .categories
        .iter()
        .any(|c| c.source_id == s && c.key == k && c.pinned);
    let (profile, tab, k) = (a.profile(), a.state.tab, k.to_owned());
    a.write(
        move |db| db.set_category_pinned(profile, s, tab, &k, !pinned),
        |a| a.reload_categories(),
    );
}

pub fn connect(win: &AppWindow) {
    let api = win.global::<BrowseApi>();
    api.on_pin_toggled(|id| {
        with_app(|a| {
            if let Some(Sel::Category(s, k)) = Sel::from_id(&id) {
                toggle_pin(a, s, &k);
            }
        });
    });
    api.on_group_new(|| {
        with_app(|a| groups::open_group_dialog(a, None, String::new(), None));
    });
    api.on_side_menu_requested(|id| {
        with_app(|a| {
            let sel = Sel::from_id(&id);
            let v = sel.as_ref().map(|s| menu_entries(a, s)).unwrap_or_default();
            a.browse.menu_sel = sel;
            if let Some(w) = a.win.upgrade() {
                w.global::<BrowseApi>()
                    .set_side_menu(ModelRc::new(VecModel::from(v)));
            }
        });
    });
    api.on_side_menu_picked(|id| {
        with_app(|a| {
            if let Some(sel) = a.browse.menu_sel.take() {
                menu_picked(a, sel, &id);
            }
        });
    });
    api.on_side_filter_edited(|_| {
        with_app(|a| a.push_sidebar());
    });
    api.on_side_filter_accepted(|| {
        with_app(|a| {
            let first = a
                .browse
                .side_rows
                .iter()
                .find(|r| r.kind == 2 || r.kind == 3);
            if let Some(sel) = first.and_then(|r| Sel::from_id(&r.id)) {
                select(a, sel);
                a.scroll_sidebar_to_selection();
            }
        });
    });
    api.on_sidebar_step(|step| {
        with_app(|a| {
            let order = a.sidebar_order();
            if step == 0 || order.is_empty() {
                return;
            }
            let at = order.iter().position(|s| Some(s) == a.state.sel.as_ref());
            let next = at.map_or(0, |i| {
                i.saturating_add_signed(step as isize).min(order.len() - 1)
            });
            if at != Some(next) {
                select(a, order[next].clone());
                a.scroll_sidebar_to_selection();
            }
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_matches_case_insensitive_substring() {
        assert!(matches("AR| BEIN SPORTS ⁸ᴷ", ""));
        assert!(matches("AR| BEIN SPORTS ⁸ᴷ", "bein"));
        assert!(matches("FR | Cinéma", "CINÉ"));
        assert!(!matches("FR | Cinema", "sport"));
    }
}
