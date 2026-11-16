//! Built-in catalog of public XMLTV guides, tagged by country (`uk` = Great Britain), `arabic` or `*` (global).
use super::country;
use crate::model::StreamGuideKey;
use std::collections::{HashMap, HashSet};

const MIN_STREAMS: usize = 5;

pub struct CatalogGuide {
    pub tags: Vec<&'static str>,
    pub name: &'static str,
    pub url: &'static str,
}

/// The built-in list of public guides, in file order.
pub fn all() -> Vec<CatalogGuide> {
    include_str!("catalog.tsv")
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| {
            let mut f = l.split('\t');
            Some(CatalogGuide {
                tags: f.next()?.split(',').collect(),
                name: f.next()?,
                url: f.next()?.trim(),
            })
        })
        .collect()
}

fn stream_tags(s: &StreamGuideKey) -> HashSet<String> {
    let mut tags = HashSet::new();
    tags.extend(s.epg_id.as_deref().and_then(country::of_channel));
    tags.extend(country::of_name(&s.name).map(|t| if t == "ar" { "arabic".into() } else { t }));
    tags
}

/// How many live streams hint at each tag.
pub fn playlist_tags(streams: &[StreamGuideKey]) -> HashMap<String, usize> {
    let mut out: HashMap<String, usize> = HashMap::new();
    for t in streams.iter().flat_map(stream_tags) {
        *out.entry(t).or_default() += 1;
    }
    out
}

/// Catalog guides worth adding: global (`*`) ones and those with a tag hinted by >= 5 streams.
pub fn suggested(streams: &[StreamGuideKey]) -> Vec<CatalogGuide> {
    let counts = playlist_tags(streams);
    all()
        .into_iter()
        .filter(|g| {
            g.tags
                .iter()
                .any(|t| *t == "*" || counts.get(*t).is_some_and(|&n| n >= MIN_STREAMS))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn streams(prefix: &str, n: usize) -> Vec<StreamGuideKey> {
        (0..n)
            .map(|i| StreamGuideKey {
                source_id: 1,
                stream_key: i.to_string(),
                epg_id: None,
                name: format!("{prefix}: Chan {i}"),
            })
            .collect()
    }

    fn has(list: &[CatalogGuide], tag: &str) -> bool {
        list.iter().any(|g| g.tags.contains(&tag))
    }

    #[test]
    fn catalog_is_sane() {
        let all = all();
        assert!(all.len() >= 200);
        assert!(
            all.iter()
                .all(|g| g.url.starts_with("https://") && !g.tags.is_empty())
        );
        let urls: HashSet<_> = all.iter().map(|g| g.url).collect();
        assert_eq!(urls.len(), all.len());
    }

    #[test]
    fn suggests_by_country_threshold() {
        let fr = suggested(&streams("FR", 5));
        assert!(has(&fr, "fr") && !has(&fr, "de"));
        assert!(!has(&suggested(&streams("DE", 4)), "de"));
    }

    #[test]
    fn arabic_prefix_and_globals() {
        let ar = suggested(&streams("AR", 5));
        assert!(ar.iter().any(|g| g.name == "iimnm Arabic"));
        assert!(!has(&ar, "ar"));
        let none = suggested(&[]);
        assert!(
            all()
                .iter()
                .filter(|g| g.tags.contains(&"*"))
                .all(|g| none.iter().any(|n| n.url == g.url))
        );
    }

    #[test]
    fn id_suffix_counts_once_per_stream() {
        let mut s = streams("FR", 1);
        s[0].epg_id = Some("tf1.fr".into());
        assert_eq!(playlist_tags(&s)["fr"], 1);
    }
}
