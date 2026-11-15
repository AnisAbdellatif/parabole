//! Sidebar rows, group/category menu, pinning, keyboard focus
//! (ports `ui/sidebar_rows.rs`, `ui/sidebar_groups.rs`, the sidebar half of `ui/nav_keys.rs`).
use super::groups;
use crate::model::MediaKind;
use crate::slint_ui::callbacks::select;
use crate::slint_ui::state::{Sel, SlintApp, with_app};
use crate::slint_ui::{AppWindow, BrowseApi, Core, MenuEntry, SideRow};
use crate::ui::sort::SortMode;
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

impl SlintApp {
    pub fn push_sidebar(&self) {
        let Some(win) = self.win.upgrade() else {
            return;
        };
        let mut v = vec![
            row(Sel::Favorites.to_id(), 1, "Favorites".into(), -1, 1, false),
            row(Sel::Recent.to_id(), 1, "Recent".into(), -1, 2, false),
        ];
        if self.state.tab != MediaKind::Live {
            let id = Sel::Continue.to_id();
            v.push(row(id, 1, "Continue watching".into(), -1, 3, false));
        }
        v.push(row(String::new(), 4, "MY GROUPS".into(), -1, 0, false));
        for g in &self.groups {
            let id = Sel::Group(g.id).to_id();
            v.push(row(id, 2, g.name.clone(), g.count as i32, 0, false));
        }
        let cats: Vec<_> = self.categories.iter().filter(|c| !c.hidden).collect();
        let pinned = cats.iter().filter(|c| c.pinned).count();
        for (title, want) in [("PINNED", true), ("", false)] {
            let mut part = cats.iter().filter(|c| c.pinned == want).peekable();
            if part.peek().is_none() {
                continue;
            }
            let title = match (want, pinned) {
                (false, 0) => "CATEGORIES",
                (false, _) => "ALL CATEGORIES",
                _ => title,
            };
            v.push(header(title));
            for c in part {
                let id = Sel::Category(c.source_id, c.key.clone()).to_id();
                v.push(row(id, 3, self.cat_label(c), c.count as i32, 0, c.pinned));
            }
        }
        self.browse.side_rows.set_vec(v);
        win.global::<Core>().set_selected_id(
            self.state
                .sel
                .as_ref()
                .map(Sel::to_id)
                .unwrap_or_default()
                .into(),
        );
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

    /// Selectable sidebar entries top to bottom, matching the normal-mode layout.
    fn sidebar_order(&self) -> Vec<Sel> {
        let mut out = vec![Sel::Favorites, Sel::Recent];
        if self.state.tab != MediaKind::Live {
            out.push(Sel::Continue);
        }
        out.extend(self.groups.iter().map(|g| Sel::Group(g.id)));
        for pinned in [true, false] {
            let cats = self
                .categories
                .iter()
                .filter(|c| c.pinned == pinned && !c.hidden);
            out.extend(cats.map(|c| Sel::Category(c.source_id, c.key.clone())));
        }
        out
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
            let res = a.db.set_category_hidden(profile, *s, tab, k, true);
            a.report(res);
            a.reload_categories();
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
            let res =
                a.db.set_setting(Some(profile), &sort_key(tab, *s, k), mode.as_str());
            a.report(res);
            a.reload_streams();
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
    let res =
        a.db.set_category_pinned(a.profile(), s, a.state.tab, k, !pinned);
    a.report(res);
    a.reload_categories();
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
