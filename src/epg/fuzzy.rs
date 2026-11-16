//! Fuzzy name matching: trigram shortlist, then word-by-word spelling tolerance.
use super::country;
use super::matching::{Candidate, norm_name};
use std::collections::{HashMap, HashSet};

const FILLER: [&str; 12] = [
    "al", "el", "de", "la", "le", "les", "du", "des", "di", "del", "and", "et",
];

/// Name words for fuzzy comparison: `matching::norm_name` split on whitespace, minus filler words.
pub fn tokens(name: &str) -> Vec<String> {
    norm_name(name)
        .split_whitespace()
        .filter(|t| !FILLER.contains(t))
        .map(str::to_string)
        .collect()
}

/// Word count before filler removal.
fn words(name: &str) -> usize {
    norm_name(name).split_whitespace().count()
}

struct Entry {
    words: usize,
    cand: usize,
    tokens: Vec<String>,
    grams: usize,
}

pub struct Index {
    entries: Vec<Entry>,
    postings: HashMap<[char; 3], Vec<usize>>,
}

fn trigrams(key: &str) -> HashSet<[char; 3]> {
    let cs: Vec<char> = format!(" {key} ").chars().collect();
    cs.windows(3).map(|w| [w[0], w[1], w[2]]).collect()
}

/// A channel id as name words: `nessma.tn@sd` -> `nessma`.
fn id_tokens(id: &str) -> Vec<String> {
    let id = id.split('@').next().unwrap_or(id);
    let base = id.rsplit_once('.').map_or(id, |(b, _)| b);
    tokens(&base.replace(['.', '_', '-'], " "))
}

impl Index {
    pub fn new(candidates: &[Candidate]) -> Self {
        let mut entries = Vec::new();
        let mut postings: HashMap<[char; 3], Vec<usize>> = HashMap::new();
        for (ci, c) in candidates.iter().enumerate() {
            if c.channel
                .split('@')
                .next()
                .unwrap_or("")
                .trim()
                .chars()
                .count()
                < 3
            {
                continue;
            }
            let id = id_tokens(&c.channel);
            let all = c.names.iter().map(|n| (tokens(n), words(n)));
            for (toks, n_words) in all.chain([(id.clone(), id.len())]) {
                let key = toks.concat();
                if key.chars().count() < 3 {
                    continue;
                }
                let grams = trigrams(&key);
                let e = entries.len();
                for g in &grams {
                    postings.entry(*g).or_default().push(e);
                }
                entries.push(Entry {
                    cand: ci,
                    words: n_words,
                    tokens: toks,
                    grams: grams.len(),
                });
            }
        }
        Index { entries, postings }
    }

    /// Index into `candidates` of the best fuzzy match for a stream name, if any.
    pub fn best(&self, stream_name: &str, candidates: &[Candidate]) -> Option<usize> {
        let toks = tokens(stream_name);
        let key = toks.concat();
        if key.chars().count() < 4 {
            return None;
        }
        let grams = trigrams(&key);
        let mut counts = vec![0u32; self.entries.len()];
        let mut touched = Vec::new();
        for g in &grams {
            for &e in self.postings.get(g).into_iter().flatten() {
                if counts[e] == 0 {
                    touched.push(e);
                }
                counts[e] += 1;
            }
        }
        let mut top: Vec<(usize, u32)> = touched.into_iter().map(|e| (e, counts[e])).collect();
        top.sort_unstable_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        top.truncate(50);
        let n_words = words(stream_name);
        let mut best: Option<(f64, usize)> = None;
        for (ei, n) in top {
            let e = &self.entries[ei];
            let dice = 2.0 * f64::from(n) / (grams.len() + e.grams) as f64;
            // a lone word has no context to absorb a typo or a dropped "al": exact only
            let lone_ok = toks.len() > 1 || (e.tokens == toks && e.words == n_words);
            if dice < 0.5
                || !lone_ok
                || !aligned(&toks, &e.tokens)
                || country::conflict(stream_name, &candidates[e.cand].channel)
            {
                continue;
            }
            let better = best.is_none_or(|(d, c)| dice > d || (dice == d && e.cand < c));
            if better {
                best = Some((dice, e.cand));
            }
        }
        best.map(|(_, c)| c)
    }
}

fn aligned(a: &[String], b: &[String]) -> bool {
    if a.is_empty() || a.len() != b.len() {
        return false;
    }
    let mut used = vec![false; b.len()];
    a.iter().all(|x| {
        let hit = (0..b.len()).find(|&j| !used[j] && close(x, &b[j]));
        hit.is_some_and(|j| {
            used[j] = true;
            true
        })
    })
}

fn close(x: &str, y: &str) -> bool {
    if x == y {
        return true;
    }
    if x.chars().any(|c| c.is_ascii_digit()) || y.chars().any(|c| c.is_ascii_digit()) {
        return false;
    }
    // one letter changes a short name ("wpix" vs "kpix"); allow only a plural there
    if x.chars().count().min(y.chars().count()) <= 5 {
        return x.strip_suffix('s') == Some(y) || y.strip_suffix('s') == Some(x);
    }
    levenshtein(x, y) <= 2
}

fn levenshtein(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut cur = vec![i + 1];
        for (j, cb) in b.iter().enumerate() {
            let sub = prev[j] + usize::from(ca != *cb);
            cur.push(sub.min(prev[j + 1] + 1).min(cur[j] + 1));
        }
        prev = cur;
    }
    prev[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cand(channel: &str, names: &[&str]) -> Candidate {
        Candidate {
            guide_id: 1,
            channel: channel.into(),
            names: names.iter().map(|s| s.to_string()).collect(),
        }
    }

    fn best(stream: &str, c: &[Candidate]) -> Option<usize> {
        Index::new(c).best(stream, c)
    }

    #[test]
    fn lone_words_need_exact() {
        assert_eq!(
            best("TV: Danger TV", &[cand("dangal.in", &["Dangal"])]),
            None
        );
        assert_eq!(best("AR: Al Ons TV", &[cand("ons.nl", &["Ons"])]), None);
        assert_eq!(
            best("MX: TLNOVELAS", &[cand("x.mx", &["TNT Novelas"])]),
            None
        );
    }

    #[test]
    fn short_words_need_exact_or_plural() {
        assert!(close("sport", "sports"));
        assert!(!close("wpix", "kpix"));
        assert!(!close("iman", "oman"));
        assert!(!close("inews", "cnews"));
        assert!(close("ejdida", "jadida"));
    }

    #[test]
    fn matches_spelling_variants() {
        let c = [cand("nessma.tn", &["Nessma El Jadida"])];
        assert_eq!(best("AR: Nessma Ejdida", &c), Some(0));
        let c = [cand("a.fr", &["beIN SPORTS MAX 4"])];
        assert_eq!(best("FR: BEIN SPORT MAX 4 HD", &c), Some(0));
        let c = [cand("a.es", &["M+ Liga de Campeones 3"])];
        assert_eq!(best("ES: M. LIGA CAMPEONES 3", &c), Some(0));
    }

    #[test]
    fn rejects_near_misses() {
        let cases = [
            ("Tunisia National 1", "x.tn", "Tunisia National 2"),
            (
                "NBC Sports California Plus",
                "x.us",
                "NBC Sports California",
            ),
            ("beIN Sports 1 FR", "x.fr", "beIN Sports 1"),
            ("Sky One", "x.uk", "Sky News"),
            ("RO: Explore", "explore.no", "Explore"),
        ];
        for (s, id, n) in cases {
            assert_eq!(best(s, &[cand(id, &[n])]), None, "{s}");
        }
    }

    #[test]
    fn ties_go_to_lower_index() {
        let c = [
            cand("a.fr", &["Canal Sport"]),
            cand("b.fr", &["Canal Sport"]),
        ];
        assert_eq!(best("Canal Sports", &c), Some(0));
    }
}
