/// ISO alpha-2 for a country name as TheSportsDB writes it (case-insensitive).
pub fn iso(name: &str) -> Option<&'static str> {
    let name = name.trim();
    TABLE
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case(name))
        .map(|(_, c)| *c)
}

const TABLE: &[(&str, &str)] = &[
    ("United Kingdom", "GB"),
    ("UK", "GB"),
    ("England", "GB"),
    ("Scotland", "GB"),
    ("Wales", "GB"),
    ("Northern Ireland", "GB"),
    ("Ireland", "IE"),
    ("USA", "US"),
    ("United States", "US"),
    ("Australia", "AU"),
    ("New Zealand", "NZ"),
    ("France", "FR"),
    ("Spain", "ES"),
    ("Mexico", "MX"),
    ("Argentina", "AR"),
    ("Colombia", "CO"),
    ("Chile", "CL"),
    ("Peru", "PE"),
    ("Uruguay", "UY"),
    ("Germany", "DE"),
    ("Austria", "AT"),
    ("Italy", "IT"),
    ("Portugal", "PT"),
    ("Brazil", "BR"),
    ("Netherlands", "NL"),
    ("Holland", "NL"),
    ("Saudi Arabia", "SA"),
    ("Egypt", "EG"),
    ("United Arab Emirates", "AE"),
    ("UAE", "AE"),
    ("Qatar", "QA"),
    ("Morocco", "MA"),
    ("Algeria", "DZ"),
    ("Tunisia", "TN"),
    ("Turkey", "TR"),
    ("Poland", "PL"),
    ("Switzerland", "CH"),
    ("Belgium", "BE"),
    ("Canada", "CA"),
    ("India", "IN"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_map_case_insensitively() {
        assert_eq!(iso("united kingdom"), Some("GB"));
        assert_eq!(iso("England"), Some("GB"));
        assert_eq!(iso("USA"), Some("US"));
        assert_eq!(iso(" Spain "), Some("ES"));
        assert_eq!(iso("Atlantis"), None);
    }
}
