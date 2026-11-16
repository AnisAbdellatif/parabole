//! Matches provider streams to guide channels: exact id, normalised id, then name.
use crate::db::alt_name;
use crate::model::{EpgLink, GuideChannel, GuideId, StreamGuideKey};
use std::collections::{HashMap, HashSet};

pub struct Candidate {
    pub guide_id: GuideId,
    pub channel: String,
    pub names: Vec<String>,
}

/// Trim, lowercase, strip a trailing `@...` suffix.
pub fn norm_id(id: &str) -> String {
    let id = id.trim().to_lowercase();
    match id.find('@') {
        Some(i) => id[..i].trim_end().to_string(),
        None => id,
    }
}

/// Normalised name without filler words; empty when nothing is left.
pub fn norm_name(name: &str) -> String {
    const NOISE: [&str; 5] = ["tv", "lq", "channel", "chaine", "the"];
    alt_name::normalize(name)
        .split_whitespace()
        .filter(|t| !NOISE.contains(t))
        .collect::<Vec<_>>()
        .join(" ")
}

/// A channel id without its last `.xx` segment, as a name (`nessma.tn` -> `nessma`).
fn id_as_name(id: &str) -> String {
    let base = id.rsplit_once('.').map_or(id, |(b, _)| b);
    norm_name(&base.replace('.', " "))
}

/// The stream's guide id, unless missing or junk (under 3 chars).
fn stream_id(s: &StreamGuideKey) -> Option<String> {
    let id = s.epg_id.as_deref()?.trim().to_lowercase();
    (id.chars().count() >= 3).then_some(id)
}

/// Maps a key to the first candidate index having it.
fn first_index<K: std::hash::Hash + Eq>(
    keys: impl Iterator<Item = (K, usize)>,
) -> HashMap<K, usize> {
    let mut m = HashMap::new();
    for (k, i) in keys {
        m.entry(k).or_insert(i);
    }
    m
}

/// Links each stream to one guide channel. `candidates` are in guide priority order (only channels that have programmes).
pub fn link(streams: &[StreamGuideKey], candidates: &[Candidate]) -> Vec<EpgLink> {
    let exact = first_index(
        candidates
            .iter()
            .enumerate()
            .map(|(i, c)| (c.channel.to_lowercase(), i)),
    );
    let by_id = first_index(
        candidates
            .iter()
            .enumerate()
            .map(|(i, c)| (norm_id(&c.channel), i)),
    );
    let by_name = first_index(candidates.iter().enumerate().flat_map(|(i, c)| {
        let own = c.names.iter().map(|n| norm_name(n));
        own.chain([id_as_name(&c.channel.to_lowercase())])
            .filter(|n| !n.is_empty())
            .map(move |n| (n, i))
    }));
    streams
        .iter()
        .filter_map(|s| {
            let id = stream_id(s);
            let hit = id
                .as_ref()
                .and_then(|id| exact.get(id).or_else(|| by_id.get(&norm_id(id))))
                .or_else(|| {
                    let n = norm_name(&s.name);
                    (!n.is_empty()).then(|| by_name.get(&n)).flatten()
                })?;
            let c = &candidates[*hit];
            Some(EpgLink {
                source_id: s.source_id,
                stream_key: s.stream_key.clone(),
                guide_id: c.guide_id,
                channel: c.channel.clone(),
            })
        })
        .collect()
}

/// Channel ids of one guide that any stream would match (to keep only those programmes while parsing).
pub fn wanted(streams: &[StreamGuideKey], channels: &[GuideChannel]) -> HashSet<String> {
    let cands: Vec<Candidate> = channels
        .iter()
        .map(|c| Candidate {
            guide_id: 0,
            channel: c.id.clone(),
            names: c.names.clone(),
        })
        .collect();
    link(streams, &cands)
        .into_iter()
        .map(|l| l.channel)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn st(key: &str, epg: Option<&str>, name: &str) -> StreamGuideKey {
        StreamGuideKey {
            source_id: 1,
            stream_key: key.into(),
            epg_id: epg.map(Into::into),
            name: name.into(),
        }
    }

    fn cand(guide_id: GuideId, channel: &str, names: &[&str]) -> Candidate {
        Candidate {
            guide_id,
            channel: channel.into(),
            names: names.iter().map(|s| s.to_string()).collect(),
        }
    }

    fn one(s: StreamGuideKey, c: &[Candidate]) -> Option<(GuideId, String)> {
        link(&[s], c).pop().map(|l| (l.guide_id, l.channel))
    }

    #[test]
    fn tiers() {
        let c = [cand(1, "tf1.fr", &["TF1"])];
        assert_eq!(one(st("a", Some(" TF1.FR "), "x"), &c).unwrap().1, "tf1.fr");
        let c = [cand(1, "hannibaltv.tn@sd", &[])];
        assert!(one(st("a", Some("hannibaltv.tn"), "x"), &c).is_some());
        let c = [cand(1, "x1", &["Canal+ HD"])];
        assert!(one(st("a", None, "FR: Canal+"), &c).is_some());
        let c = [cand(1, "nessma.tn", &[])];
        assert!(one(st("a", None, "AR: Nessma TV"), &c).is_some());
        assert!(one(st("a", None, "Other"), &c).is_none());
    }

    #[test]
    fn precedence_and_priority() {
        let c = [cand(1, "a1", &["Foo"]), cand(2, "foo.fr", &["Bar"])];
        let s = st("a", Some("foo.fr"), "Foo");
        assert_eq!(one(s, &c).unwrap().0, 2);
        let c = [cand(1, "a1", &["Foo"]), cand(2, "a2", &["Foo"])];
        assert_eq!(one(st("a", None, "Foo"), &c).unwrap().0, 1);
    }

    #[test]
    fn junk_ids_ignored() {
        let c = [cand(1, "ts", &[])];
        assert!(one(st("a", Some("ts"), "Zzz"), &c).is_none());
    }

    #[test]
    fn real_cases() {
        let c = [cand(1, "hannibaltv.tn@sd", &["Hannibal TV"])];
        assert!(one(st("a", Some("hannibaltv.tn"), "AR: Hannibal ◉"), &c).is_some());
        assert!(one(st("a", None, "AR: Hannibal ◉"), &c).is_some());
        // different words ("Ejdida" vs "El Jadida"): no match, needs a manual link
        let c = [cand(1, "nessma.tn", &["Nessma El Jadida"])];
        assert!(one(st("a", Some("ts"), "AR: Nessma Ejdida"), &c).is_none());
        // "al hiwar tunisia" != "al hiwar"
        let c = [cand(1, "alhiwar.tn", &["Al Hiwar TV"])];
        assert!(one(st("a", None, "AR: Al Hiwar Tunisia"), &c).is_none());
    }

    #[test]
    fn wanted_reuses_link() {
        let ch = [
            GuideChannel {
                id: "a.fr".into(),
                names: vec!["Alpha".into()],
            },
            GuideChannel {
                id: "b.fr".into(),
                names: vec!["Beta".into()],
            },
        ];
        let w = wanted(&[st("1", None, "Alpha HD")], &ch);
        assert_eq!(w, HashSet::from(["a.fr".to_string()]));
    }
}
