//! 365scores ids. Edit these tables when ids change; nothing else should need to.

/// ISO 3166-1 alpha-2 → 365scores `userCountryId` (TV channels are returned per user country).
pub const COUNTRIES: &[(&str, i64)] = &[
    ("GB", 1),
    ("IE", 9),
    ("US", 18),
    ("CA", 66),
    ("AU", 36),
    ("FR", 5),
    ("BE", 16),
    ("CH", 15),
    ("ES", 2),
    ("MX", 31),
    ("AR", 10),
    ("CO", 109),
    ("CL", 28),
    ("PE", 112),
    ("DE", 4),
    ("AT", 20),
    ("IT", 3),
    ("PT", 11),
    ("BR", 21),
    ("NL", 7),
    ("SA", 122),
    ("EG", 131),
    ("AE", 124),
    ("QA", 115),
    ("MA", 127),
    ("TR", 12),
    ("PL", 37),
];

pub fn country_id(iso: &str) -> Option<i64> {
    COUNTRIES
        .iter()
        .find(|(c, _)| c.eq_ignore_ascii_case(iso))
        .map(|(_, id)| *id)
}
