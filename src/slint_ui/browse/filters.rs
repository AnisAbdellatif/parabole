//! Sort selector and search-filter chips (ports `ui/sort.rs`, `ui/search_filter.rs`).
use super::search_filter::MEDIA_CHIPS;
use super::sort::SortMode;
use crate::model::MediaKind;
use crate::slint_ui::state::{Sel, SlintApp, with_app};
use crate::slint_ui::{AppWindow, BrowseApi, ChipItem, MenuEntry};
use slint::{ComponentHandle, ModelRc, VecModel};

fn sort_key(a: &SlintApp) -> Option<String> {
    match &a.state.sel {
        Some(Sel::Category(s, k)) if a.search_applied.is_empty() => {
            Some(format!("sort:{s}:{}:{k}", a.state.tab.as_str()))
        }
        _ => None,
    }
}

pub fn load_sort(a: &SlintApp) -> SortMode {
    let saved = sort_key(a).and_then(|k| a.db.get_setting(Some(a.profile()), &k).ok().flatten());
    saved.map_or(SortMode::Provider, |v| SortMode::parse(&v))
}

/// Search results are grid cards only when every result is a movie/series.
pub fn grid_view(a: &SlintApp) -> bool {
    let tab = a.state.tab;
    if a.search_applied.is_empty() {
        return tab != MediaKind::Live;
    }
    matches!(a.browse.filter.media(tab), Some(m) if m != MediaKind::Live)
}

fn chip(id: String, label: &str, on: bool) -> ChipItem {
    ChipItem {
        id: id.into(),
        label: label.into(),
        on,
    }
}

fn entry(id: &str, label: &str) -> MenuEntry {
    MenuEntry {
        id: id.into(),
        label: label.into(),
        kind: 0,
    }
}

impl SlintApp {
    /// Pushes sort selector, chips and the grid/list switch.
    pub fn push_browse_header(&self) {
        let Some(win) = self.win.upgrade() else {
            return;
        };
        let api = win.global::<BrowseApi>();
        api.set_grid_view(grid_view(self));
        api.set_sort_visible(sort_key(self).is_some());
        api.set_sort_label(self.browse.sort.label().into());
        let items: Vec<MenuEntry> = SortMode::ALL
            .iter()
            .map(|m| entry(&format!("sort:{}", m.as_str()), m.label()))
            .collect();
        api.set_sort_items(ModelRc::new(VecModel::from(items)));
        api.set_chips(ModelRc::new(VecModel::from(self.chips())));
    }

    fn chips(&self) -> Vec<ChipItem> {
        if self.search_applied.is_empty() {
            return Vec::new();
        }
        let media = self.browse.filter.media(self.state.tab);
        let source = self.browse.filter.source(self.state.source);
        let mut v: Vec<ChipItem> = MEDIA_CHIPS
            .iter()
            .map(|(label, m)| chip(media_id(*m), label, media == *m))
            .collect();
        if self.sources.len() > 1 {
            v.push(chip("s:all".into(), "All sources", source.is_none()));
            for s in &self.sources {
                v.push(chip(format!("s:{}", s.id), &s.name, source == Some(s.id)));
            }
        }
        v
    }
}

fn media_id(m: Option<MediaKind>) -> String {
    format!("m:{}", m.map_or("all", |m| m.as_str()))
}

fn media_from(s: &str) -> Option<Option<MediaKind>> {
    match s {
        "all" => Some(None),
        "live" => Some(Some(MediaKind::Live)),
        "movie" => Some(Some(MediaKind::Movie)),
        "series" => Some(Some(MediaKind::Series)),
        _ => None,
    }
}

pub fn connect(win: &AppWindow) {
    let api = win.global::<BrowseApi>();
    api.on_sort_picked(|id| {
        with_app(|a| {
            let Some(key) = sort_key(a) else { return };
            let mode = SortMode::parse(id.strip_prefix("sort:").unwrap_or_default());
            if mode == a.browse.sort {
                return;
            }
            let res = a.db.set_setting(Some(a.profile()), &key, mode.as_str());
            a.report(res);
            a.reload_streams();
        });
    });
    api.on_chip_clicked(|id| {
        with_app(|a| {
            if let Some(m) = id.strip_prefix("m:").and_then(media_from) {
                a.browse.filter.media = Some(m);
            } else if let Some(s) = id.strip_prefix("s:") {
                a.browse.filter.source = Some(s.parse().ok());
            } else {
                return;
            }
            a.reload_streams();
        });
    });
}
