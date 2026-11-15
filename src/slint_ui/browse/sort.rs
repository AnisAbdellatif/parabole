use crate::model::StreamRow;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SortMode {
    #[default]
    Provider,
    Number,
    Name,
}

impl SortMode {
    pub(crate) const ALL: [SortMode; 3] = [SortMode::Provider, SortMode::Number, SortMode::Name];

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            SortMode::Provider => "provider",
            SortMode::Number => "number",
            SortMode::Name => "name",
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            SortMode::Provider => "Provider order",
            SortMode::Number => "Number",
            SortMode::Name => "A–Z",
        }
    }

    pub(crate) fn parse(s: &str) -> Self {
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
            progress: None,
            resume_episode: None,
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
