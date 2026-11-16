//! Country hints: from a channel id suffix (`tf1.fr`) and a stream name prefix (`FR: TF1`).
use crate::db::alt_name::strip_country;

fn code(c: &str) -> Option<String> {
    let c = c.to_ascii_lowercase();
    let ok = c.len() == 2 && c.chars().all(|ch| ch.is_ascii_lowercase());
    ok.then(|| if c == "gb" { "uk".into() } else { c })
}

/// Two-letter country of a guide channel id: `bbcone.uk@hd` -> `uk`; `gb` reads as `uk`.
pub fn of_channel(id: &str) -> Option<String> {
    let base = id.split('@').next()?;
    code(base.rsplit_once('.')?.1)
}

/// Two-letter prefix of a stream name: `FR: TF1` -> `fr`. `AR` here means Arabic, not Argentina.
pub fn of_name(name: &str) -> Option<String> {
    let rest = strip_country(name);
    if rest.len() == name.len() {
        return None;
    }
    let prefix = name[..name.len() - rest.len()].trim_matches(|c: char| !c.is_ascii_alphabetic());
    code(prefix)
}

/// True when the stream's name prefix names one country and the channel id another.
pub fn conflict(stream_name: &str, channel_id: &str) -> bool {
    match (of_name(stream_name), of_channel(channel_id)) {
        (Some(s), Some(c)) => s != c && s != "ar" && s != "tv",
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hints() {
        assert_eq!(of_channel("BBCOne.gb@HD").as_deref(), Some("uk"));
        assert_eq!(of_channel("51767"), None);
        assert_eq!(of_channel("x.bein"), None);
        assert_eq!(of_name("|FR| TF1").as_deref(), Some("fr"));
        assert_eq!(of_name("VIP: AD Sports"), None);
        assert_eq!(of_name("TF1"), None);
    }

    #[test]
    fn conflicts() {
        assert!(conflict("RO: Explore", "viasatexplore.no"));
        assert!(!conflict("UK: BBC One", "bbcone.gb"));
        assert!(!conflict("AR: Hannibal", "hannibaltv.tn"));
        assert!(!conflict("Explore", "viasatexplore.no"));
    }
}
