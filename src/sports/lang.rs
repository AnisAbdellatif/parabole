//! Language inference for broadcasts and the language → countries table.

pub struct Language {
    pub code: &'static str,
    pub name: &'static str,
    /// Countries queried for channels when the user picks this language (ISO alpha-2).
    pub countries: &'static [&'static str],
}

pub const LANGUAGES: &[Language] = &[
    Language {
        code: "en",
        name: "English",
        countries: &["GB", "IE", "US", "CA", "AU"],
    },
    Language {
        code: "fr",
        name: "French",
        countries: &["FR", "BE", "CH", "CA"],
    },
    Language {
        code: "es",
        name: "Spanish",
        countries: &["ES", "MX", "AR", "CO", "CL", "PE", "US"],
    },
    Language {
        code: "de",
        name: "German",
        countries: &["DE", "AT", "CH"],
    },
    Language {
        code: "it",
        name: "Italian",
        countries: &["IT", "CH"],
    },
    Language {
        code: "pt",
        name: "Portuguese",
        countries: &["PT", "BR"],
    },
    Language {
        code: "ar",
        name: "Arabic",
        countries: &["SA", "EG", "AE", "QA", "MA"],
    },
    Language {
        code: "nl",
        name: "Dutch",
        countries: &["NL", "BE"],
    },
    Language {
        code: "tr",
        name: "Turkish",
        countries: &["TR"],
    },
    Language {
        code: "pl",
        name: "Polish",
        countries: &["PL"],
    },
];

// Multilingual countries (CH, BE, CA, IN) are deliberately absent.
const COUNTRY_LANG: &[(&str, &str)] = &[
    ("GB", "en"),
    ("IE", "en"),
    ("US", "en"),
    ("AU", "en"),
    ("NZ", "en"),
    ("FR", "fr"),
    ("ES", "es"),
    ("MX", "es"),
    ("AR", "es"),
    ("CO", "es"),
    ("CL", "es"),
    ("PE", "es"),
    ("UY", "es"),
    ("DE", "de"),
    ("AT", "de"),
    ("IT", "it"),
    ("PT", "pt"),
    ("BR", "pt"),
    ("NL", "nl"),
    ("SA", "ar"),
    ("EG", "ar"),
    ("AE", "ar"),
    ("QA", "ar"),
    ("MA", "ar"),
    ("DZ", "ar"),
    ("TN", "ar"),
    ("TR", "tr"),
    ("PL", "pl"),
];

const HINTS: &[(&str, &str)] = &[
    ("arabic", "ar"),
    ("(ar)", "ar"),
    ("english", "en"),
    ("(en)", "en"),
    ("french", "fr"),
    ("(fr)", "fr"),
    ("deportes", "es"),
    ("tudn", "es"),
    ("telemundo", "es"),
    ("univision", "es"),
    ("unimás", "es"),
    ("unimas", "es"),
    ("(es)", "es"),
    ("german", "de"),
    ("(de)", "de"),
];

const SUFFIXES: &[&str] = &["EN", "FR", "ES", "DE", "IT", "PT", "AR", "NL", "TR", "PL"];

pub fn for_channel(channel: &str, country: &str) -> Option<String> {
    let lower = channel.to_lowercase();
    if let Some((_, code)) = HINTS.iter().find(|(needle, _)| lower.contains(needle)) {
        return Some((*code).to_string());
    }
    if let Some(last) = channel.split_whitespace().last()
        && SUFFIXES.contains(&last)
    {
        return Some(last.to_lowercase());
    }
    let country = country.to_ascii_uppercase();
    COUNTRY_LANG
        .iter()
        .find(|(c, _)| *c == country)
        .map(|(_, code)| (*code).to_string())
}

pub fn countries_for(langs: &[String]) -> Vec<&'static str> {
    let mut out: Vec<&'static str> = Vec::new();
    for code in langs {
        if let Some(l) = LANGUAGES.iter().find(|l| l.code == code) {
            for c in l.countries {
                if !out.contains(c) {
                    out.push(c);
                }
            }
        }
    }
    out
}

pub fn name(code: &str) -> &'static str {
    LANGUAGES
        .iter()
        .find(|l| l.code == code)
        .map_or("Other", |l| l.name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hint_wins_over_country() {
        assert_eq!(
            for_channel("beIN SPORTS 1 (Arabic)", "FR").as_deref(),
            Some("ar")
        );
    }

    #[test]
    fn suffix_token() {
        assert_eq!(for_channel("beIN SPORTS 1 EN", "QA").as_deref(), Some("en"));
    }

    #[test]
    fn country_fallback() {
        assert_eq!(for_channel("Canal+", "FR").as_deref(), Some("fr"));
    }

    #[test]
    fn multilingual_and_unknown_are_none() {
        assert_eq!(for_channel("RTS 2", "CH"), None);
        assert_eq!(for_channel("Foo", "ZZ"), None);
    }

    #[test]
    fn countries_deduped_in_order() {
        assert_eq!(
            countries_for(&["en".into(), "es".into()]),
            [
                "GB", "IE", "US", "CA", "AU", "ES", "MX", "AR", "CO", "CL", "PE"
            ]
        );
    }

    #[test]
    fn unknown_name_is_other() {
        assert_eq!(name("xx"), "Other");
        assert_eq!(name("fr"), "French");
    }
}
