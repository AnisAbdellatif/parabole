//! Chooses the confident IMDb match for a cleaned stream name.
use super::title::{Cleaned, normalize};
use crate::model::MediaKind;

#[derive(Clone, Debug, PartialEq)]
pub struct Candidate {
    pub id: String,
    pub title: String,
    pub year: Option<i64>,
    /// IMDb `qid`, e.g. `movie`, `tvSeries`; empty when unknown.
    pub kind: String,
    pub poster: Option<String>,
}

fn allowed(media: MediaKind, kind: &str) -> bool {
    match media {
        MediaKind::Movie => matches!(kind, "movie" | "tvMovie" | "video"),
        MediaKind::Series => matches!(kind, "tvSeries" | "tvMiniSeries"),
        MediaKind::Live => false,
    }
}

pub fn pick<'a>(cands: &'a [Candidate], want: &Cleaned, media: MediaKind) -> Option<&'a Candidate> {
    let wanted = normalize(&want.title);
    let mut matches = cands.iter().filter(|c| {
        allowed(media, &c.kind)
            && normalize(&c.title) == wanted
            && match (c.year, want.year) {
                (Some(a), Some(b)) => (a - b).abs() <= 1,
                _ => true,
            }
    });
    let first = matches.next()?;
    // without a year, several same-titled titles are remakes: refuse to guess
    (want.year.is_some() || matches.next().is_none()).then_some(first)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cand(id: &str, title: &str, year: Option<i64>, kind: &str) -> Candidate {
        Candidate {
            id: id.into(),
            title: title.into(),
            year,
            kind: kind.into(),
            poster: None,
        }
    }

    fn want(title: &str, year: Option<i64>) -> Cleaned {
        Cleaned {
            title: title.into(),
            year,
        }
    }

    #[test]
    fn year_picks_first_close_match() {
        let c = [
            cand("tt1", "Dune", Some(1984), "movie"),
            cand("tt2", "Dune", Some(2021), "movie"),
        ];
        let p = pick(&c, &want("Dune", Some(2022)), MediaKind::Movie);
        assert_eq!(p.map(|c| c.id.as_str()), Some("tt2"));
        assert_eq!(pick(&c, &want("Dune", Some(2000)), MediaKind::Movie), None);
    }

    #[test]
    fn no_year_needs_single_match() {
        let one = [cand("tt1", "The Matrix", Some(1999), "movie")];
        assert!(pick(&one, &want("the matrix", None), MediaKind::Movie).is_some());
        let two = [
            cand("tt1", "Dune", Some(1984), "movie"),
            cand("tt2", "Dune", Some(2021), "movie"),
        ];
        assert_eq!(pick(&two, &want("Dune", None), MediaKind::Movie), None);
    }

    #[test]
    fn kind_filters() {
        let c = [
            cand("tt1", "Fargo", Some(1996), "movie"),
            cand("tt2", "Fargo", Some(2014), "tvSeries"),
            cand("tt3", "Fargo", None, ""),
        ];
        let m = pick(&c, &want("Fargo", None), MediaKind::Movie);
        assert_eq!(m.map(|c| c.id.as_str()), Some("tt1"));
        let s = pick(&c, &want("Fargo", None), MediaKind::Series);
        assert_eq!(s.map(|c| c.id.as_str()), Some("tt2"));
        assert_eq!(pick(&c, &want("Fargo", None), MediaKind::Live), None);
    }

    #[test]
    fn title_must_match_exactly() {
        let c = [cand("tt1", "The Matrix Reloaded", Some(2003), "movie")];
        assert_eq!(pick(&c, &want("The Matrix", None), MediaKind::Movie), None);
    }
}
