use super::entry::Draft;
use crate::model::{MediaKind, ParsedCategory, ParsedStream};
use std::collections::{HashMap, HashSet};

const KEY_SEP: char = '\u{1f}';

/// Assigns unique `ext_key`s and derives categories in first-appearance order.
pub fn finish(drafts: Vec<Draft>) -> (Vec<ParsedCategory>, Vec<ParsedStream>) {
    let mut id_counts: HashMap<(MediaKind, &str), u32> = HashMap::new();
    for d in &drafts {
        if let Some(id) = d.tvg_id.as_deref() {
            *id_counts.entry((d.stream.media, id)).or_default() += 1;
        }
    }
    let unique_ids: HashSet<(MediaKind, String)> = id_counts
        .into_iter()
        .filter(|(_, n)| *n == 1)
        .map(|((media, id), _)| (media, id.to_string()))
        .collect();

    let mut seen_groups = HashSet::new();
    let mut occurrences: HashMap<(MediaKind, String), u32> = HashMap::new();
    let mut categories = Vec::new();
    let mut streams = Vec::with_capacity(drafts.len());
    for d in drafts {
        let Draft {
            mut stream,
            tvg_id,
            group,
        } = d;
        let media = stream.media;
        if seen_groups.insert((media, group.clone())) {
            categories.push(ParsedCategory {
                media,
                ext_key: group.clone(),
                name: group.clone(),
            });
        }
        let mut key = match tvg_id {
            Some(id) if unique_ids.contains(&(media, id.clone())) => id,
            _ => format!("{}{KEY_SEP}{group}", stream.name),
        };
        let n = occurrences.entry((media, key.clone())).or_default();
        *n += 1;
        if *n > 1 {
            key = format!("{key}#{n}");
        }
        stream.ext_key = key;
        streams.push(stream);
    }
    (categories, streams)
}
