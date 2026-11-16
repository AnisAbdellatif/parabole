use crate::meta::pick::Candidate;
use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde::Deserialize;
use url::Url;

const BASE: &str = "https://v3.sg.media-imdb.com/suggestion/";

#[derive(Deserialize)]
struct Response {
    #[serde(default)]
    d: Vec<Entry>,
}

#[derive(Deserialize)]
struct Entry {
    id: String,
    l: Option<String>,
    y: Option<i64>,
    qid: Option<String>,
    i: Option<Image>,
}

#[derive(Deserialize)]
struct Image {
    #[serde(rename = "imageUrl")]
    image_url: Option<String>,
}

/// Keeps title entries (`tt…`) only; persons and companies are dropped.
pub fn parse(json: &str) -> Result<Vec<Candidate>> {
    let r: Response = serde_json::from_str(json).context("parse IMDb suggestions")?;
    Ok(r.d
        .into_iter()
        .filter(|e| e.id.starts_with("tt"))
        .filter_map(|e| {
            Some(Candidate {
                id: e.id,
                title: e.l?,
                year: e.y,
                kind: e.qid.unwrap_or_default(),
                poster: e.i.and_then(|i| i.image_url),
            })
        })
        .collect())
}

fn url(query: &str) -> Result<Url> {
    let first = query
        .chars()
        .next()
        .map(|c| c.to_ascii_lowercase())
        .filter(char::is_ascii_alphanumeric)
        .unwrap_or('x');
    let mut url = Url::parse(BASE).context("IMDb base url")?;
    url.path_segments_mut()
        .map_err(|_| anyhow::anyhow!("IMDb base url cannot be a base"))?
        .pop_if_empty()
        .push(&first.to_string())
        .push(&format!("{query}.json"));
    Ok(url)
}

pub fn search(client: &Client, query: &str) -> Result<Vec<Candidate>> {
    let body = client
        .get(url(query)?)
        .send()
        .and_then(|r| r.error_for_status())
        .and_then(|r| r.text())
        .map_err(reqwest::Error::without_url)
        .context("IMDb search")?;
    parse(&body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_fixture() {
        let c = parse(include_str!(
            "../../../tests/fixtures/meta/imdb_suggest.json"
        ))
        .unwrap();
        assert_eq!(c.len(), 2, "person entry dropped");
        assert_eq!(c[0].id, "tt0133093");
        assert_eq!(c[0].title, "The Matrix");
        assert_eq!(c[0].year, Some(1999));
        assert_eq!(c[0].kind, "movie");
        assert!(c[0].poster.as_deref().unwrap().contains("media-amazon"));
        assert_eq!(c[1].year, None);
        assert_eq!(c[1].kind, "");
        assert_eq!(c[1].poster, None);
    }

    #[test]
    fn garbage_is_an_error() {
        assert!(parse("<html>").is_err());
    }

    #[test]
    fn builds_url() {
        let u = url("The Matrix: Reloaded/2").unwrap();
        assert_eq!(
            u.as_str(),
            "https://v3.sg.media-imdb.com/suggestion/t/The%20Matrix:%20Reloaded%2F2.json"
        );
        assert!(url("Émile").unwrap().as_str().contains("/suggestion/x/"));
    }
}
