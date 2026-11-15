//! Search chips (media kind, source) and how they resolve against the tab and top-bar source.
use crate::model::{MediaKind, SourceId};

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
