//! Quality a channel name claims through tags like "FHD", "4K" or "ᴴᴰ".

use crate::db::alt_name::strip_country;

/// Longest first so "ᶠᴴᴰ" is removed before the "ᴴᴰ" it contains.
const SUPERSCRIPT: [(&str, i64); 3] = [("ᵁᴴᴰ", 2160), ("ᶠᴴᴰ", 1080), ("ᴴᴰ", 720)];

/// `(height, fps)` a lowercase tag stands for.
fn tag(token: &str) -> Option<(Option<i64>, Option<f64>)> {
    let height = match token {
        "8k" => 4320,
        "4k" | "uhd" | "2160p" | "2160" => 2160,
        "fhd" | "1080p" | "1080" | "1080i" => 1080,
        "hd" | "720p" | "720" => 720,
        "sd" | "576p" | "480p" => 480,
        "50fps" => return Some((None, Some(50.0))),
        "60fps" => return Some((None, Some(60.0))),
        _ => return None,
    };
    Some((Some(height), None))
}

/// Claimed height and fps from the name's tags; the highest tag wins.
pub fn claimed(name: &str) -> (Option<i64>, Option<f64>) {
    let mut height = SUPERSCRIPT
        .iter()
        .filter(|(s, _)| name.contains(s))
        .map(|(_, h)| *h)
        .max();
    let mut fps: Option<f64> = None;
    let lower = name.to_lowercase();
    for token in lower.split(|c: char| !c.is_alphanumeric()) {
        if let Some((h, f)) = tag(token) {
            height = height.max(h);
            fps = match (fps, f) {
                (Some(a), Some(b)) => Some(a.max(b)),
                (a, b) => a.or(b),
            };
        }
    }
    (height, fps)
}

/// The name without country prefix, quality tags and lone symbols.
pub fn display_name(name: &str) -> String {
    let mut rest = strip_country(name).to_string();
    for (s, _) in SUPERSCRIPT {
        rest = rest.replace(s, " ");
    }
    let kept: Vec<&str> = rest
        .split_whitespace()
        .filter(|t| t.chars().any(char::is_alphanumeric))
        .filter(|t| {
            let bare = t.trim_matches(|c: char| !c.is_alphanumeric());
            tag(&bare.to_lowercase()).is_none()
        })
        .collect();
    if kept.is_empty() {
        name.trim().to_string()
    } else {
        kept.join(" ")
    }
}
