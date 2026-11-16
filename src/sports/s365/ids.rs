//! 365scores ids. Edit these tables when ids change; nothing else should need to.

/// Competition id → display name. This is the "popular competitions" allowlist.
pub const COMPETITIONS: &[(i64, &str)] = &[
    (7, "Premier League"),
    (11, "LaLiga"),
    (17, "Serie A"),
    (25, "Bundesliga"),
    (35, "Ligue 1"),
    (572, "Champions League"),
    (573, "Europa League"),
    (7685, "Conference League"),
    (472, "UEFA Super Cup"),
    (7016, "Nations League"),
    (5930, "World Cup"),
    (5421, "World Cup qualifiers (UEFA)"),
    (613, "World Cup qualifiers (CONMEBOL)"),
    (6316, "Euro"),
    (6071, "Euro qualifiers"),
    (595, "Copa América"),
    (102, "Copa Libertadores"),
    (8, "FA Cup"),
    (13, "Copa del Rey"),
    (20, "Coppa Italia"),
    (28, "DFB-Pokal"),
    (37, "Coupe de France"),
];

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

pub fn competition_name(id: i64) -> Option<&'static str> {
    COMPETITIONS.iter().find(|(i, _)| *i == id).map(|(_, n)| *n)
}

pub fn country_id(iso: &str) -> Option<i64> {
    COUNTRIES
        .iter()
        .find(|(c, _)| c.eq_ignore_ascii_case(iso))
        .map(|(_, id)| *id)
}
