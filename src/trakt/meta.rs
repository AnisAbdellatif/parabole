use super::client::get_json;
use crate::meta::imdb::sized;
use crate::meta::title::{Cleaned, normalize};
use crate::model::{MediaKind, Meta};
use anyhow::Result;
use reqwest::blocking::Client;
use serde_json::Value;
use url::form_urlencoded::byte_serialize;

type Found = Option<(Meta, Option<i64>)>;

fn kind(media: MediaKind) -> Option<&'static str> {
    match media {
        MediaKind::Movie => Some("movie"),
        MediaKind::Series => Some("show"),
        MediaKind::Live => None,
    }
}

fn text(v: &Value, key: &str) -> Option<String> {
    v.get(key)?
        .as_str()
        .filter(|s| !s.is_empty())
        .map(String::from)
}

fn title_case(slug: &str) -> String {
    slug.split('-')
        .map(|w| {
            let mut c = w.chars();
            c.next()
                .map(|f| f.to_uppercase().chain(c).collect::<String>())
                .unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn poster(v: &Value) -> Option<String> {
    let url = v.get("images")?.get("poster")?.get(0)?.as_str()?;
    let url = if url.contains("://") {
        url.to_string()
    } else {
        format!("https://{url}")
    };
    Some(sized(&url, 600))
}

/// One element of a Trakt search result; the second value is the Trakt id.
pub fn parse_item(v: &Value, media: MediaKind) -> Found {
    let o = v.get(kind(media)?)?;
    let ids = o.get("ids");
    let meta = Meta {
        origin: "Trakt".into(),
        imdb_id: ids.and_then(|i| text(i, "imdb")),
        title: text(o, "title")?,
        year: o.get("year").and_then(Value::as_i64),
        rating: o.get("rating").and_then(Value::as_f64).map(|r| r as f32),
        votes: o.get("votes").and_then(Value::as_i64),
        plot: text(o, "overview"),
        poster: poster(o),
        genres: o
            .get("genres")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(title_case)
            .collect(),
        runtime_min: o.get("runtime").and_then(Value::as_i64),
        certificate: text(o, "certification"),
        directors: Vec::new(),
        cast: Vec::new(),
    };
    Some((
        meta,
        ids.and_then(|i| i.get("trakt")).and_then(Value::as_i64),
    ))
}

pub fn by_imdb(client: &Client, imdb_id: &str, media: MediaKind) -> Result<Found> {
    let Some(kind) = kind(media) else {
        return Ok(None);
    };
    let path = format!("/search/imdb/{imdb_id}?type={kind}&extended=full,images");
    let v = get_json(client, &path)?;
    Ok(v.as_array()
        .and_then(|a| a.first())
        .and_then(|i| parse_item(i, media)))
}

pub fn search(client: &Client, want: &Cleaned, media: MediaKind) -> Result<Found> {
    let Some(kind) = kind(media) else {
        return Ok(None);
    };
    let q: String = byte_serialize(want.title.as_bytes()).collect();
    let v = get_json(
        client,
        &format!("/search/{kind}?query={q}&extended=full,images"),
    )?;
    Ok(first_match(&v, want, media))
}

fn first_match(v: &Value, want: &Cleaned, media: MediaKind) -> Found {
    let wanted = normalize(&want.title);
    v.as_array()?
        .iter()
        .filter_map(|i| parse_item(i, media))
        .find(|(m, _)| {
            normalize(&m.title) == wanted
                && match (m.year, want.year) {
                    (Some(a), Some(b)) => (a - b).abs() <= 1,
                    _ => true,
                }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Value {
        serde_json::from_str(include_str!(
            "../../tests/fixtures/meta/trakt_search_movie.json"
        ))
        .unwrap()
    }

    fn want(title: &str, year: Option<i64>) -> Cleaned {
        Cleaned {
            title: title.into(),
            year,
        }
    }

    #[test]
    fn parses_item() {
        let v = fixture();
        let (m, trakt) = parse_item(&v[0], MediaKind::Movie).unwrap();
        assert_eq!(trakt, Some(481));
        assert_eq!(m.origin, "Trakt");
        assert_eq!(m.imdb_id.as_deref(), Some("tt0133093"));
        assert_eq!(m.title, "The Matrix");
        assert_eq!(m.year, Some(1999));
        assert_eq!(m.runtime_min, Some(136));
        assert_eq!(m.genres, ["Action", "Science Fiction"]);
        assert_eq!(m.certificate.as_deref(), Some("R"));
        assert_eq!(
            m.poster.as_deref(),
            Some("https://media.trakt.tv/images/movies/000/000/481/posters/thumb/matrix.jpg")
        );
        assert!(m.rating.unwrap() > 8.0);
        assert!(parse_item(&v[0], MediaKind::Series).is_none());
        assert!(parse_item(&v[0], MediaKind::Live).is_none());
    }

    #[test]
    fn keeps_scheme_of_full_urls() {
        let v = fixture();
        let (m, _) = parse_item(&v[1], MediaKind::Movie).unwrap();
        assert!(m.poster.unwrap().starts_with("https://walter.trakt.tv/"));
    }

    #[test]
    fn first_match_checks_title_and_year() {
        let v = fixture();
        let hit = first_match(&v, &want("the matrix", Some(1999)), MediaKind::Movie);
        assert_eq!(hit.unwrap().1, Some(481));
        let hit = first_match(
            &v,
            &want("The Matrix Reloaded", Some(2003)),
            MediaKind::Movie,
        );
        assert_eq!(hit.unwrap().1, Some(482));
        assert!(first_match(&v, &want("The Matrix", Some(2010)), MediaKind::Movie).is_none());
        assert!(first_match(&v, &want("Matrix", None), MediaKind::Movie).is_none());
    }

    #[test]
    fn unconfigured_search_fails() {
        let client = Client::new();
        assert!(search(&client, &want("x", None), MediaKind::Movie).is_err());
        assert!(by_imdb(&client, "tt1", MediaKind::Movie).is_err());
    }
}
