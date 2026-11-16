//! Quality scan category picker: which live categories the background scanner probes.
use super::{AppWindow, DialogApi};
use crate::model::{MediaKind, SourceId};
use crate::slint_ui::state::{SlintApp, with_app};
use crate::slint_ui::{ScanApi, ScanCat};
use slint::{ComponentHandle, ModelRc, VecModel};
use std::collections::{HashMap, HashSet};

pub(super) struct Entry {
    source: SourceId,
    key: String,
    label: String,
    count: i64,
    visible: bool,
    checked: bool,
}

fn matches(label: &str, filter: &str) -> bool {
    let label = label.to_lowercase();
    filter
        .to_lowercase()
        .split_whitespace()
        .all(|w| label.contains(w))
}

impl SlintApp {
    pub(crate) fn open_scan_dialog(&mut self) {
        let profile = self.profile();
        let loaded = self
            .db
            .categories(profile, None, MediaKind::Live, true)
            .and_then(|c| Ok((c, self.db.scan_selection_or_default(profile)?)));
        let (cats, sel) = match loaded {
            Ok(v) => v,
            Err(e) => return self.toast(format!("{e:#}")),
        };
        let sel: HashSet<(SourceId, String)> = sel.into_iter().collect();
        let names: HashMap<SourceId, &str> = self
            .sources
            .iter()
            .map(|s| (s.id, s.name.as_str()))
            .collect();
        let multi = self.sources.len() > 1;
        let entries = cats
            .into_iter()
            .map(|c| Entry {
                checked: sel.contains(&(c.source_id, c.key.clone())),
                label: match names.get(&c.source_id) {
                    Some(src) if multi => format!("{src} · {}", c.name),
                    _ => c.name,
                },
                source: c.source_id,
                key: c.key,
                count: c.count,
                visible: !c.hidden,
            })
            .collect();
        self.dialogs.scan = entries;
        if let Some(w) = self.win.upgrade() {
            w.global::<ScanApi>().set_filter("".into());
            w.global::<DialogApi>().set_open("scan".into());
        }
        self.push_scan_rows();
    }

    fn scan_filter(&self) -> String {
        let w = self.win.upgrade();
        w.map(|w| w.global::<ScanApi>().get_filter().to_string())
            .unwrap_or_default()
    }

    fn push_scan_rows(&self) {
        let Some(w) = self.win.upgrade() else { return };
        let filter = self.scan_filter();
        let rows: Vec<ScanCat> = (self.dialogs.scan.iter().enumerate())
            .filter(|(_, e)| matches(&e.label, &filter))
            .map(|(i, e)| ScanCat {
                idx: i as i32,
                label: e.label.as_str().into(),
                count: e.count.try_into().unwrap_or(i32::MAX),
                checked: e.checked,
            })
            .collect();
        let picked = self.dialogs.scan.iter().filter(|e| e.checked).count();
        let total = self.dialogs.scan.len();
        let api = w.global::<ScanApi>();
        api.set_summary(format!("{picked} of {total} selected").into());
        api.set_rows(ModelRc::new(VecModel::from(rows)));
    }

    fn scan_preset(&mut self, preset: i32) {
        let filter = self.scan_filter();
        for e in &mut self.dialogs.scan {
            match preset {
                0 => e.checked = e.visible,
                // All / None act on the filtered rows only.
                p if matches(&e.label, &filter) => e.checked = p == 1,
                _ => {}
            }
        }
        self.push_scan_rows();
    }

    fn scan_toggled(&mut self, idx: i32) {
        if let Some(e) = self.dialogs.scan.get_mut(idx as usize) {
            e.checked = !e.checked;
        }
        // Only the summary changes; re-pushing rows would reset the list scroll.
        if let Some(w) = self.win.upgrade() {
            let picked = self.dialogs.scan.iter().filter(|e| e.checked).count();
            let total = self.dialogs.scan.len();
            let text = format!("{picked} of {total} selected");
            w.global::<ScanApi>().set_summary(text.into());
        }
    }

    fn save_scan_dialog(&mut self) {
        let picked: Vec<(SourceId, String)> = (self.dialogs.scan.iter())
            .filter(|e| e.checked)
            .map(|e| (e.source, e.key.clone()))
            .collect();
        let profile = self.profile();
        let res = self.db.set_scan_categories(profile, &picked);
        self.report(res);
        self.dialogs.scan.clear();
        self.close_dialog();
    }
}

pub fn connect(win: &AppWindow) {
    let api = win.global::<ScanApi>();
    api.on_filter_edited(|_| {
        with_app(|a| a.push_scan_rows());
    });
    api.on_toggled(|i| {
        with_app(|a| a.scan_toggled(i));
    });
    api.on_preset(|p| {
        with_app(|a| a.scan_preset(p));
    });
    api.on_save(|| {
        with_app(|a| a.save_scan_dialog());
    });
}

#[cfg(test)]
mod tests {
    #[test]
    fn filter_matches_all_words() {
        assert!(super::matches("France · Sports HD", "sport fr"));
        assert!(super::matches("Anything", ""));
        assert!(!super::matches("France · News", "sport"));
    }
}
