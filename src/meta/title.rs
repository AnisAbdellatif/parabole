//! Turns IPTV stream names into searchable titles.

#[derive(Debug, PartialEq)]
pub struct Cleaned {
    pub title: String,
    pub year: Option<i64>,
}

const TAGS: &[&str] = &[
    "4k", "uhd", "fhd", "hd", "sd", "hdr", "1080p", "720p", "2160p", "multi", "multisub", "vostfr",
    "vf", "vo", "sub", "dub", "hevc", "x265", "h265", "3d",
];

/// Lowercase, `&` as "and", everything else non-alphanumeric as single spaces.
pub fn normalize(s: &str) -> String {
    let mut out = String::new();
    for c in s.to_lowercase().replace('&', " and ").chars() {
        out.push(if c.is_alphanumeric() { c } else { ' ' });
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn clean(name: &str) -> Cleaned {
    let mut s = name.trim();
    for _ in 0..3 {
        let next = strip_prefix(s);
        if next.len() == s.len() {
            break;
        }
        s = next;
    }
    let s = strip_brackets(s);
    let mut words: Vec<&str> = s.split_whitespace().filter(|w| !is_tag(w)).collect();
    let mut year = None;
    if let Some(i) = words.iter().position(|w| paren_year(w).is_some()) {
        year = paren_year(words[i]);
        words.remove(i);
    } else if words.len() > 1 {
        // "Blade Runner 2049" also loses its 2049 here; worker::meta retries with it put back
        if let Some(y) = words.last().and_then(|w| plain_year(w)) {
            year = Some(y);
            words.pop();
        }
    }
    let title = words.join(" ");
    let title = title.trim_matches([' ', '-', '|', ':']).to_string();
    Cleaned { title, year }
}

fn is_tag(w: &str) -> bool {
    let w = w
        .trim_matches(|c: char| !c.is_alphanumeric())
        .to_lowercase();
    TAGS.contains(&w.as_str())
}

fn in_range(y: i64) -> Option<i64> {
    (1900..=2099).contains(&y).then_some(y)
}

fn paren_year(w: &str) -> Option<i64> {
    let inner = w
        .trim_end_matches([',', '.'])
        .strip_prefix('(')?
        .strip_suffix(')')?;
    plain_year(inner)
}

fn plain_year(w: &str) -> Option<i64> {
    if w.len() != 4 || !w.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    in_range(w.parse().ok()?)
}

fn strip_brackets(s: &str) -> String {
    let mut out = String::new();
    let mut depth = 0;
    for c in s.chars() {
        match c {
            '[' => depth += 1,
            ']' if depth > 0 => depth -= 1,
            _ if depth == 0 => out.push(c),
            _ => {}
        }
    }
    out
}

/// Drops one leading `|XX|`, `[XX]` or `XX - ` / `XX| ` / `XX: ` style language prefix.
fn strip_prefix(s: &str) -> &str {
    for (open, close) in [('|', '|'), ('[', ']')] {
        if let Some(rest) = s.strip_prefix(open)
            && let Some(end) = rest.find(close)
        {
            return rest[end + 1..].trim_start();
        }
    }
    let run_end = s
        .find(|c: char| !(c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-'))
        .unwrap_or(s.len());
    let run = &s[..run_end];
    let code = run.trim_end_matches('-');
    let valid = !code.is_empty()
        && code.split('-').all(|p| (2..=4).contains(&p.len()))
        && code.bytes().any(|b| b.is_ascii_uppercase());
    if !valid {
        return s;
    }
    let after = &s[code.len()..];
    let rest = after.trim_start();
    let spaced = after.len() != rest.len();
    match rest.chars().next() {
        Some('|') => rest[1..].trim_start(),
        Some('-') if spaced || rest[1..].starts_with(' ') => rest[1..].trim_start(),
        Some(':') if rest[1..].starts_with(' ') => rest[1..].trim_start(),
        _ => s,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(name: &str) -> (String, Option<i64>) {
        let r = clean(name);
        (r.title, r.year)
    }

    #[test]
    fn cleans_names() {
        let cases: &[(&str, &str, Option<i64>)] = &[
            ("EN - The Matrix (1999) 4K", "The Matrix", Some(1999)),
            (
                "|FR| Le Dîner de cons 1998 FHD",
                "Le Dîner de cons",
                Some(1998),
            ),
            ("Breaking Bad [MULTI-SUB]", "Breaking Bad", None),
            (
                "4K-NF - Dune: Part Two (2024)",
                "Dune: Part Two",
                Some(2024),
            ),
            ("Inception", "Inception", None),
            ("FR| Amélie", "Amélie", None),
            ("|NL| Oppenheimer HDR", "Oppenheimer", None),
            ("[MULTI] Joker (2019)", "Joker", Some(2019)),
            ("WALL-E", "WALL-E", None),
            ("The Office US - HD", "The Office US", None),
            ("2012", "2012", None),
            ("Heat 1995", "Heat", Some(1995)),
            ("  Se7en   (1995)  1080p ", "Se7en", Some(1995)),
            ("AR: Gladiator VOSTFR", "Gladiator", None),
            ("Game of Thrones x265 UHD", "Game of Thrones", None),
            ("Movie 1899", "Movie 1899", None),
        ];
        for (name, title, year) in cases {
            assert_eq!(c(name), (title.to_string(), *year), "{name}");
        }
    }

    #[test]
    fn normalizes() {
        assert_eq!(normalize("Dune: Part Two!"), "dune part two");
        assert_eq!(normalize("Tom & Jerry"), "tom and jerry");
        assert_eq!(normalize("  A   B "), "a b");
    }
}
