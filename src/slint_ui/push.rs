//! Copies app state into the `Core` global's properties.
use super::Core;
use super::state::{Sel, SlintApp};
use crate::model::StreamRow;
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};

/// Shortcut modifier prefix shown in labels: `MOD_LABEL` + "F".
pub const MOD_LABEL: &str = if cfg!(target_os = "macos") {
    "⌘"
} else {
    "Ctrl+"
};

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
        let name = names.get(idx).cloned().unwrap_or_default();
        let initials: String = name
            .split_whitespace()
            .filter_map(|w| w.chars().next())
            .take(2)
            .collect();
        core.set_profile_initials(initials.to_uppercase().into());
        core.set_profile_name(name);
        core.set_profile_index(idx as i32);
        core.set_profiles(ModelRc::new(VecModel::from(names)));
    }

    pub fn push_source_filter(&self) {
        let Some(win) = self.win.upgrade() else {
            return;
        };
        let mut names: Vec<SharedString> = vec!["All sources".into()];
        names.extend(
            self.sources
                .iter()
                .map(|s| SharedString::from(s.name.as_str())),
        );
        let idx = self
            .sources
            .iter()
            .position(|s| Some(s.id) == self.state.source);
        let core = win.global::<Core>();
        core.set_source_names(ModelRc::new(VecModel::from(names)));
        core.set_source_index(idx.map_or(0, |i| i as i32 + 1));
    }

    pub fn push_streams(&mut self, rows: Vec<StreamRow>) {
        let n = rows.len();
        let prefix = self.state.source.is_none() && self.sources.len() > 1;
        let names = self
            .sources
            .iter()
            .map(|s| (s.id, s.name.clone()))
            .collect();
        self.streams.set_badges(self.row_badges(&rows));
        self.streams.set_epg(super::epg::row_epg(self, &rows));
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
                Some(Sel::Quality(b)) => crate::smart::Bucket::from_index(*b)
                    .map_or("", |b| b.label())
                    .into(),
                None => String::new(),
            }
        };
        core.set_list_title(title.into());
        core.set_list_subtitle(count(n, "item", "items").into());
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
            format!("{} synced", count(n, "source", "sources"))
        } else {
            "Syncing…".into()
        };
        core.set_status_sources(text.into());
        let items: i64 = self.categories.iter().map(|c| c.count).sum();
        core.set_status_items(
            format!(
                "{} in {}",
                count(items, "item", "items"),
                count(self.categories.len(), "category", "categories")
            )
            .into(),
        );
        self.push_empty();
    }
}

/// "1 item", "3 items".
fn count<T: std::fmt::Display + PartialEq + From<u8>>(n: T, one: &str, many: &str) -> String {
    let word = if n == T::from(1) { one } else { many };
    format!("{n} {word}")
}

#[cfg(test)]
mod tests {
    use super::count;

    #[test]
    fn count_picks_singular_only_for_one() {
        assert_eq!(count(1, "item", "items"), "1 item");
        assert_eq!(count(0, "item", "items"), "0 items");
        assert_eq!(count(2, "category", "categories"), "2 categories");
    }
}
