//! Search chips (media kind, source) and how they resolve against the tab and top-bar source.
use super::App;
use super::kit;
use crate::model::{MediaKind, SourceId};
use egui::{Ui, vec2};

/// `None` = follow the current tab / top-bar source; `Some(x)` = chip override (`x: None` = all).
#[derive(Default)]
pub struct SearchFilter {
    pub(crate) media: Option<Option<MediaKind>>,
    pub(crate) source: Option<Option<SourceId>>,
}

impl SearchFilter {
    pub fn media(&self, tab: MediaKind) -> Option<MediaKind> {
        self.media.unwrap_or(Some(tab))
    }

    pub fn source(&self, bar: Option<SourceId>) -> Option<SourceId> {
        self.source.unwrap_or(bar)
    }
}

pub(crate) const MEDIA_CHIPS: [(&str, Option<MediaKind>); 4] = [
    ("All", None),
    ("Live", Some(MediaKind::Live)),
    ("Movies", Some(MediaKind::Movie)),
    ("Series", Some(MediaKind::Series)),
];

fn chip(ui: &mut Ui, label: &str, on: bool) -> bool {
    let size = vec2(kit::pill_width(ui, label), 28.0);
    kit::pill(ui, label, size, on, true).clicked()
}

impl App {
    /// Search results are grid cards only when every result is a movie/series.
    pub(super) fn grid_view(&self) -> bool {
        let tab = self.state.tab;
        if self.search.applied.is_empty() {
            return tab != MediaKind::Live;
        }
        matches!(self.search.filter.media(tab), Some(m) if m != MediaKind::Live)
    }

    pub(super) fn search_chips(&mut self, ui: &mut Ui) {
        let media = self.search.filter.media(self.state.tab);
        let source = self.search.filter.source(self.state.source);
        let (mut new_media, mut new_source) = (None, None);
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
            for (label, m) in MEDIA_CHIPS {
                if chip(ui, label, media == m) {
                    new_media = Some(m);
                }
            }
            if self.sources.len() > 1 {
                ui.add_space(10.0);
                if chip(ui, "All sources", source.is_none()) {
                    new_source = Some(None);
                }
                for s in &self.sources {
                    if chip(ui, &s.name, source == Some(s.id)) {
                        new_source = Some(Some(s.id));
                    }
                }
            }
        });
        if new_media.is_some() || new_source.is_some() {
            if new_media.is_some() {
                self.search.filter.media = new_media;
            }
            if new_source.is_some() {
                self.search.filter.source = new_source;
            }
            self.selected = 0;
            self.reload_streams();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_follow_tab_and_top_bar() {
        let f = SearchFilter::default();
        assert_eq!(f.media(MediaKind::Movie), Some(MediaKind::Movie));
        assert_eq!(f.source(Some(3)), Some(3));
    }

    #[test]
    fn chips_override_tab_and_top_bar() {
        let f = SearchFilter {
            media: Some(None),
            source: Some(Some(7)),
        };
        assert_eq!(f.media(MediaKind::Live), None);
        assert_eq!(f.source(Some(3)), Some(7));
        let all = SearchFilter {
            media: Some(Some(MediaKind::Series)),
            source: Some(None),
        };
        assert_eq!(all.media(MediaKind::Live), Some(MediaKind::Series));
        assert_eq!(all.source(Some(3)), None);
    }
}
