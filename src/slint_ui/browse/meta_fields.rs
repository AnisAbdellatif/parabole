//! Merges online details over provider details into the labelled fields of the info view.
use crate::model::{MediaInfo, MediaKind, Meta};

/// 950 / 12K / 2.3M
pub fn votes(n: i64) -> String {
    match n {
        ..1_000 => n.to_string(),
        1_000..1_000_000 => format!("{}K", n / 1_000),
        _ => format!("{:.1}M", n as f64 / 1_000_000.0),
    }
}

/// 136 -> "2h 16m", 120 -> "2h", 45 -> "45m"
pub fn runtime(min: i64) -> String {
    match (min / 60, min % 60) {
        (0, m) => format!("{m}m"),
        (h, 0) => format!("{h}h"),
        (h, m) => format!("{h}h {m}m"),
    }
}

fn rating(meta: &Meta) -> Option<String> {
    let r = meta.rating?;
    let mut s = format!("★ {r:.1} {}", meta.origin);
    if let Some(v) = meta.votes {
        s.push_str(&format!(" · {} votes", votes(v)));
    }
    Some(s)
}

fn list(items: &[String]) -> Option<String> {
    (!items.is_empty()).then(|| items.join(", "))
}

/// Online values win, except the release date; labels are shown in this order.
pub fn fields(
    provider: Option<&MediaInfo>,
    meta: Option<&Meta>,
    media: MediaKind,
) -> Vec<(&'static str, String)> {
    let p = |f: fn(&MediaInfo) -> &Option<String>| provider.and_then(|i| f(i).clone());
    let from_meta = |f: fn(&Meta) -> Option<String>| meta.and_then(f);
    let director_label = match (media, meta.is_some_and(|m| !m.directors.is_empty())) {
        (MediaKind::Series, true) => "Creator",
        _ => "Director",
    };
    [
        ("Rating", from_meta(rating).or_else(|| p(|i| &i.rating))),
        (
            "Genre",
            from_meta(|m| list(&m.genres)).or_else(|| p(|i| &i.genre)),
        ),
        (
            "Released",
            p(|i| &i.release_date).or_else(|| from_meta(|m| m.year.map(|y| y.to_string()))),
        ),
        (
            "Duration",
            from_meta(|m| m.runtime_min.map(runtime)).or_else(|| p(|i| &i.duration)),
        ),
        ("Rated", from_meta(|m| m.certificate.clone())),
        (
            director_label,
            from_meta(|m| list(&m.directors)).or_else(|| p(|i| &i.director)),
        ),
        (
            "Cast",
            from_meta(|m| list(&m.cast)).or_else(|| p(|i| &i.cast)),
        ),
        (
            "Plot",
            from_meta(|m| m.plot.clone()).or_else(|| p(|i| &i.plot)),
        ),
    ]
    .into_iter()
    .filter_map(|(l, v)| v.map(|v| (l, v)))
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meta() -> Meta {
        Meta {
            origin: "IMDb".into(),
            title: "Breaking Bad".into(),
            year: Some(2008),
            rating: Some(9.5),
            votes: Some(2_300_000),
            plot: Some("Online plot".into()),
            genres: vec!["Crime".into(), "Drama".into()],
            runtime_min: Some(48),
            certificate: Some("TV-MA".into()),
            directors: vec!["Vince Gilligan".into()],
            cast: vec!["Bryan Cranston".into()],
            ..Default::default()
        }
    }

    fn provider() -> MediaInfo {
        MediaInfo {
            plot: Some("Provider plot".into()),
            genre: Some("Thriller".into()),
            release_date: Some("2008-01-20".into()),
            rating: Some("8".into()),
            director: Some("Someone".into()),
            ..Default::default()
        }
    }

    fn get<'a>(f: &'a [(&'static str, String)], label: &str) -> Option<&'a str> {
        f.iter().find(|(l, _)| *l == label).map(|(_, v)| v.as_str())
    }

    #[test]
    fn formats_votes() {
        assert_eq!(votes(950), "950");
        assert_eq!(votes(12_345), "12K");
        assert_eq!(votes(2_300_000), "2.3M");
    }

    #[test]
    fn formats_runtime() {
        assert_eq!(runtime(136), "2h 16m");
        assert_eq!(runtime(45), "45m");
        assert_eq!(runtime(120), "2h");
    }

    #[test]
    fn meta_wins_except_release_date() {
        let f = fields(Some(&provider()), Some(&meta()), MediaKind::Series);
        assert_eq!(get(&f, "Rating"), Some("★ 9.5 IMDb · 2.3M votes"));
        assert_eq!(get(&f, "Genre"), Some("Crime, Drama"));
        assert_eq!(get(&f, "Released"), Some("2008-01-20"));
        assert_eq!(get(&f, "Duration"), Some("48m"));
        assert_eq!(get(&f, "Rated"), Some("TV-MA"));
        assert_eq!(get(&f, "Creator"), Some("Vince Gilligan"));
        assert_eq!(get(&f, "Director"), None);
        assert_eq!(get(&f, "Plot"), Some("Online plot"));
        assert_eq!(f[0].0, "Rating");
    }

    #[test]
    fn falls_back_to_provider_and_meta_only() {
        let f = fields(Some(&provider()), None, MediaKind::Movie);
        assert_eq!(get(&f, "Rating"), Some("8"));
        assert_eq!(get(&f, "Director"), Some("Someone"));
        assert_eq!(get(&f, "Rated"), None);

        let f = fields(None, Some(&meta()), MediaKind::Movie);
        assert_eq!(get(&f, "Released"), Some("2008"));
        assert_eq!(get(&f, "Director"), Some("Vince Gilligan"));

        let bare = Meta {
            origin: "Trakt".into(),
            rating: Some(7.0),
            ..Default::default()
        };
        let f = fields(Some(&provider()), Some(&bare), MediaKind::Series);
        assert_eq!(get(&f, "Rating"), Some("★ 7.0 Trakt"));
        assert_eq!(get(&f, "Genre"), Some("Thriller"));
        assert_eq!(get(&f, "Director"), Some("Someone"));
        assert!(fields(None, None, MediaKind::Movie).is_empty());
    }
}
