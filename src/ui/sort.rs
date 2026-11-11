use super::App;
use super::state::Sel;
use crate::model::StreamRow;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SortMode {
    #[default]
    Provider,
    Number,
    Name,
}

impl SortMode {
    const ALL: [SortMode; 3] = [SortMode::Provider, SortMode::Number, SortMode::Name];

    fn as_str(self) -> &'static str {
        match self {
            SortMode::Provider => "provider",
            SortMode::Number => "number",
            SortMode::Name => "name",
        }
    }

    fn label(self) -> &'static str {
        match self {
            SortMode::Provider => "Provider order",
            SortMode::Number => "Number",
            SortMode::Name => "A–Z",
        }
    }

    fn parse(s: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|m| m.as_str() == s)
            .unwrap_or_default()
    }

    /// Stable; channels without a number go last.
    pub fn apply(self, rows: &mut [StreamRow]) {
        match self {
            SortMode::Provider => {}
            SortMode::Number => rows.sort_by_key(|r| (r.num.is_none(), r.num)),
            SortMode::Name => rows.sort_by_cached_key(|r| r.name.to_lowercase()),
        }
    }
}

impl App {
    fn sort_key(&self) -> Option<String> {
        match &self.state.sel {
            Some(Sel::Category(s, k)) if self.search.applied.is_empty() => {
                Some(format!("sort:{s}:{}:{k}", self.state.tab.as_str()))
            }
            _ => None,
        }
    }

    pub(super) fn load_sort(&self) -> SortMode {
        let saved = self
            .sort_key()
            .and_then(|k| self.db.get_setting(Some(self.profile()), &k).ok().flatten());
        saved.map_or(SortMode::Provider, |v| SortMode::parse(&v))
    }

    pub(super) fn sort_selector(&mut self, ui: &mut egui::Ui) {
        let Some(key) = self.sort_key() else {
            return;
        };
        let mut mode = self.sort;
        egui::ComboBox::from_id_salt("sort_mode")
            .selected_text(mode.label())
            .show_ui(ui, |ui| {
                for m in SortMode::ALL {
                    ui.selectable_value(&mut mode, m, m.label());
                }
            });
        if mode != self.sort {
            let res = self
                .db
                .set_setting(Some(self.profile()), &key, mode.as_str());
            self.report(res);
            self.reload_streams();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::MediaKind;

    fn row(name: &str, num: Option<i64>) -> StreamRow {
        StreamRow {
            id: 0,
            source_id: 1,
            media: MediaKind::Live,
            key: name.into(),
            name: name.into(),
            num,
            logo: None,
            favorite: false,
            hidden: false,
            missing: false,
        }
    }

    fn order(mode: SortMode) -> Vec<String> {
        let mut rows = vec![row("b", None), row("C", Some(1)), row("a", Some(2))];
        mode.apply(&mut rows);
        rows.into_iter().map(|r| r.name).collect()
    }

    #[test]
    fn sorts() {
        assert_eq!(order(SortMode::Provider), ["b", "C", "a"]);
        assert_eq!(order(SortMode::Number), ["C", "a", "b"]);
        assert_eq!(order(SortMode::Name), ["a", "b", "C"]);
    }
}
