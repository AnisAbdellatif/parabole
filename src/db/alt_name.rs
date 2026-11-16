//! Channel-name normalization used to match the same channel across sources.

const QUALITY: [&str; 14] = [
    "hd", "fhd", "uhd", "sd", "raw", "4k", "8k", "hevc", "h265", "hq", "1080p", "720p", "2160p",
    "50fps",
];

/// Folds accents and superscript letters (`ᴴᴰ`) to plain lowercase; the bool marks superscripts.
fn fold(c: char) -> (char, bool) {
    let plain = match c {
        'à'..='å' | 'À'..='Å' => 'a',
        'ç' | 'Ç' => 'c',
        'è'..='ë' | 'È'..='Ë' => 'e',
        'ì'..='ï' | 'Ì'..='Ï' => 'i',
        'ñ' | 'Ñ' => 'n',
        'ò'..='ö' | 'ø' | 'Ò'..='Ö' | 'Ø' => 'o',
        'ù'..='ü' | 'Ù'..='Ü' => 'u',
        'ý' | 'ÿ' | 'Ý' => 'y',
        _ => {
            let sup = match c {
                'ᴬ' | 'ᵃ' => 'a',
                'ᴮ' | 'ᵇ' => 'b',
                'ᶜ' => 'c',
                'ᴰ' | 'ᵈ' => 'd',
                'ᴱ' | 'ᵉ' => 'e',
                'ᶠ' => 'f',
                'ᴳ' | 'ᵍ' => 'g',
                'ᴴ' | 'ʰ' => 'h',
                'ᴵ' | 'ⁱ' => 'i',
                'ᴷ' | 'ᵏ' => 'k',
                'ᴸ' | 'ˡ' => 'l',
                'ᴹ' | 'ᵐ' => 'm',
                'ᴺ' | 'ⁿ' => 'n',
                'ᴼ' | 'ᵒ' => 'o',
                'ᴾ' | 'ᵖ' => 'p',
                'ᴿ' | 'ʳ' => 'r',
                'ˢ' => 's',
                'ᵀ' | 'ᵗ' => 't',
                'ᵁ' | 'ᵘ' => 'u',
                'ᵂ' | 'ʷ' => 'w',
                'ˣ' => 'x',
                '⁰' => '0',
                '¹' => '1',
                '²' => '2',
                '³' => '3',
                '⁴'..='⁹' => char::from_digit(c as u32 - '⁴' as u32 + 4, 10).unwrap_or(' '),
                _ => return (c.to_ascii_lowercase(), false),
            };
            return (sup, true);
        }
    };
    (plain.to_ascii_lowercase(), false)
}

/// Drops a leading country tag such as `US|`, `FR:` or `|UK|`.
pub fn strip_country(name: &str) -> &str {
    let trimmed = name.trim_start_matches(['|', '[', '(', ' ']);
    match trimmed.find(['|', ':', ']', ')']) {
        Some(i)
            if (2..=3).contains(&i) && trimmed[..i].chars().all(|c| c.is_ascii_alphabetic()) =>
        {
            trimmed[i + 1..].trim_start_matches(['|', ' '])
        }
        _ => name,
    }
}

/// Case/accent/punctuation-insensitive name without quality tags and country prefix.
/// Empty when nothing identifying is left.
pub fn normalize(name: &str) -> String {
    let mut folded = String::new();
    let mut prev_sup = false;
    for c in strip_country(name).chars() {
        let (f, sup) = fold(c);
        if sup != prev_sup {
            folded.push(' ');
        }
        prev_sup = sup;
        folded.push(if f.is_alphanumeric() { f } else { ' ' });
    }
    let kept: Vec<&str> = folded
        .split_whitespace()
        .filter(|t| !QUALITY.contains(t))
        .collect();
    kept.join(" ")
}

#[cfg(test)]
mod tests {
    use super::normalize;

    #[test]
    fn strips_quality_country_case_accents() {
        assert_eq!(normalize("US| CNN HD"), "cnn");
        assert_eq!(normalize("FR: TF1 FHD"), "tf1");
        assert_eq!(normalize("|UK| BBC One ᴴᴰ"), "bbc one");
        assert_eq!(normalize("TF1ᴴᴰ"), "tf1");
        assert_eq!(normalize("Canal+ 4K"), "canal");
        assert_eq!(normalize("Été-Sport  (RAW)"), "ete sport");
        assert_eq!(normalize("ESPN ᴿᴬᵂ ⁴ᴷ"), "espn");
    }

    #[test]
    fn keeps_real_names() {
        assert_eq!(normalize("Cloud 9"), "cloud 9");
        assert_eq!(normalize("A: Team"), "a team");
        assert_eq!(normalize("HD"), "");
    }
}
