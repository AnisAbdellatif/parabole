//! Pairs the same match across providers so one provider's channels can be added to another's match.
use crate::model::Match;

const KICKOFF_SLACK_S: i64 = 15 * 60;
const STOP_WORDS: &[&str] = &[
    "fc", "cf", "ac", "sc", "afc", "cd", "sd", "ud", "club", "the",
];

/// Appends the broadcasts of each `extra` match to the `base` match it pairs with; unpaired extras are dropped.
pub fn attach(base: &mut [Match], extra: &[Match]) {
    for e in extra {
        let Some(b) = base.iter_mut().find(|b| {
            (b.kickoff_utc - e.kickoff_utc).abs() <= KICKOFF_SLACK_S
                && same_team(&b.home, &e.home)
                && same_team(&b.away, &e.away)
        }) else {
            continue;
        };
        for bc in &e.broadcasts {
            let dup = b.broadcasts.iter().any(|x| {
                x.country == bc.country && x.channel.to_lowercase() == bc.channel.to_lowercase()
            });
            if !dup {
                b.broadcasts.push(bc.clone());
            }
        }
    }
}

// shortcut: word-overlap team matching; upgrade to a per-team alias table if real mismatches show up
pub fn same_team(a: &str, b: &str) -> bool {
    let wb = words(b);
    words(a).iter().any(|w| wb.contains(w))
}

/// Lowercase, accents folded ("Atlético" → "atletico"), everything else non-alphanumeric → space.
pub fn simplify(text: &str) -> String {
    let mut folded = String::new();
    for c in text.to_lowercase().chars() {
        match fold(c) {
            Some(s) => folded.push_str(s),
            None if c.is_ascii_alphanumeric() => folded.push(c),
            None => folded.push(' '),
        }
    }
    folded
}

fn words(name: &str) -> Vec<String> {
    simplify(name)
        .split_whitespace()
        .filter(|w| w.len() >= 3 && !STOP_WORDS.contains(w))
        .map(str::to_string)
        .collect()
}

fn fold(c: char) -> Option<&'static str> {
    Some(match c {
        'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' => "a",
        'ç' | 'ć' | 'č' => "c",
        'è' | 'é' | 'ê' | 'ë' => "e",
        'ì' | 'í' | 'î' | 'ï' | 'ı' => "i",
        'ñ' => "n",
        'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' => "o",
        'ù' | 'ú' | 'û' | 'ü' => "u",
        'ý' | 'ÿ' => "y",
        'ß' => "ss",
        'ł' => "l",
        'ś' | 'š' | 'ş' => "s",
        'ž' => "z",
        'ğ' => "g",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Broadcast, MatchStatus};

    fn m(home: &str, away: &str, kickoff: i64, channels: &[(&str, &str)]) -> Match {
        Match {
            key: "k".into(),
            competition: String::new(),
            home: home.into(),
            away: away.into(),
            kickoff_utc: kickoff,
            status: MatchStatus::Upcoming,
            broadcasts: channels
                .iter()
                .map(|(ch, co)| Broadcast {
                    channel: (*ch).into(),
                    country: (*co).into(),
                    lang: None,
                    provider: "p".into(),
                })
                .collect(),
        }
    }

    #[test]
    fn team_names() {
        assert!(same_team("Slavia Praha", "Slavia Prague"));
        assert!(same_team("Atlético Madrid", "Atletico Madrid"));
        assert!(same_team("Real Madrid", "Real Sociedad"));
        assert!(!same_team("Getafe", "Girona"));
    }

    #[test]
    fn attaches_when_both_teams_pair() {
        let mut base = [m("Real Madrid", "Getafe", 1000, &[])];
        attach(
            &mut base,
            &[m("Real Madrid CF", "Getafe CF", 1300, &[("Sky", "GB")])],
        );
        assert_eq!(base[0].broadcasts.len(), 1);
    }

    #[test]
    fn different_away_not_attached() {
        let mut base = [m("Real Madrid", "Getafe", 1000, &[])];
        attach(
            &mut base,
            &[m("Real Sociedad", "Girona", 1000, &[("Sky", "GB")])],
        );
        assert!(base[0].broadcasts.is_empty());
    }

    #[test]
    fn kickoff_too_far_not_attached() {
        let mut base = [m("Real Madrid", "Getafe", 1000, &[])];
        attach(
            &mut base,
            &[m("Real Madrid", "Getafe", 1000 + 20 * 60, &[("Sky", "GB")])],
        );
        assert!(base[0].broadcasts.is_empty());
    }

    #[test]
    fn duplicates_skipped_per_country() {
        let mut base = [m("Real Madrid", "Getafe", 1000, &[("Sky Sports", "GB")])];
        attach(
            &mut base,
            &[m(
                "Real Madrid",
                "Getafe",
                1000,
                &[("sky sports", "GB"), ("Sky Sports", "IE")],
            )],
        );
        assert_eq!(base[0].broadcasts.len(), 2);
        assert_eq!(base[0].broadcasts[1].country, "IE");
    }
}
