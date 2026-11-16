use super::sized;
use crate::model::Meta;
use anyhow::{Context, Result, bail};
use reqwest::blocking::Client;
use serde::Deserialize;

const ENDPOINT: &str = "https://caching.graphql.imdb.com/";
const POSTER_WIDTH: u32 = 600;

#[derive(Deserialize)]
struct Response {
    data: Option<Data>,
    errors: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct Data {
    title: Option<Title>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Title {
    title_text: Option<Text>,
    release_year: Option<Year>,
    ratings_summary: Option<Ratings>,
    plot: Option<Plot>,
    primary_image: Option<Image>,
    runtime: Option<Runtime>,
    genres: Option<Genres>,
    certificate: Option<Certificate>,
    principal_credits: Option<Vec<Credits>>,
}

#[derive(Deserialize)]
struct Text {
    text: Option<String>,
}

#[derive(Deserialize)]
struct Year {
    year: Option<i64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Ratings {
    aggregate_rating: Option<f32>,
    vote_count: Option<i64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Plot {
    plot_text: Option<PlotText>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlotText {
    plain_text: Option<String>,
}

#[derive(Deserialize)]
struct Image {
    url: Option<String>,
}

#[derive(Deserialize)]
struct Runtime {
    seconds: Option<i64>,
}

#[derive(Deserialize)]
struct Genres {
    genres: Option<Vec<Text>>,
}

#[derive(Deserialize)]
struct Certificate {
    rating: Option<String>,
}

#[derive(Deserialize)]
struct Credits {
    category: Option<Category>,
    credits: Option<Vec<Credit>>,
}

#[derive(Deserialize)]
struct Category {
    id: Option<String>,
}

#[derive(Deserialize)]
struct Credit {
    name: Option<Name>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Name {
    name_text: Option<Text>,
}

fn names(credits: &[Credits], category: &str) -> Vec<String> {
    credits
        .iter()
        .filter(|c| c.category.as_ref().and_then(|c| c.id.as_deref()) == Some(category))
        .flat_map(|c| c.credits.iter().flatten())
        .filter_map(|c| c.name.as_ref()?.name_text.as_ref()?.text.clone())
        .collect()
}

pub fn parse(json: &str) -> Result<Meta> {
    let r: Response = serde_json::from_str(json).context("parse IMDb details")?;
    let Some(t) = r.data.and_then(|d| d.title) else {
        match r.errors {
            Some(e) => bail!("IMDb returned errors: {e}"),
            None => bail!("IMDb has no such title"),
        }
    };
    let credits = t.principal_credits.unwrap_or_default();
    let mut directors = names(&credits, "director");
    if directors.is_empty() {
        directors = names(&credits, "creator");
    }
    let ratings = t.ratings_summary;
    Ok(Meta {
        origin: "IMDb".into(),
        imdb_id: None,
        title: t.title_text.and_then(|t| t.text).unwrap_or_default(),
        year: t.release_year.and_then(|y| y.year),
        rating: ratings.as_ref().and_then(|r| r.aggregate_rating),
        votes: ratings.and_then(|r| r.vote_count),
        plot: t.plot.and_then(|p| p.plot_text?.plain_text),
        poster: t
            .primary_image
            .and_then(|i| i.url)
            .map(|u| sized(&u, POSTER_WIDTH)),
        genres: t
            .genres
            .and_then(|g| g.genres)
            .into_iter()
            .flatten()
            .filter_map(|g| g.text)
            .collect(),
        runtime_min: t.runtime.and_then(|r| r.seconds).map(|s| s / 60),
        certificate: t.certificate.and_then(|c| c.rating),
        directors,
        cast: names(&credits, "cast"),
    })
}

const QUERY: &str = "{ title(id:\"ID\") { titleText{text} titleType{id} releaseYear{year endYear} \
    ratingsSummary{aggregateRating voteCount} plot{plotText{plainText}} primaryImage{url} \
    runtime{seconds} genres{genres{text}} certificate{rating} \
    principalCredits{category{id} credits(limit:3){name{nameText{text}}}} } }";

fn valid_id(id: &str) -> bool {
    id.strip_prefix("tt")
        .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
}

pub fn fetch(client: &Client, id: &str) -> Result<Meta> {
    if !valid_id(id) {
        bail!("not an IMDb title id");
    }
    let body = client
        .post(ENDPOINT)
        .header("content-type", "application/json")
        .header("x-imdb-client-name", "imdb-web-next")
        .body(serde_json::json!({ "query": QUERY.replace("ID", id) }).to_string())
        .send()
        .and_then(|r| r.error_for_status())
        .and_then(|r| r.text())
        .map_err(reqwest::Error::without_url)
        .context("IMDb details")?;
    let mut meta = parse(&body)?;
    meta.imdb_id = Some(id.to_string());
    Ok(meta)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_movie() {
        let m = parse(include_str!(
            "../../../tests/fixtures/meta/imdb_details_movie.json"
        ))
        .unwrap();
        assert_eq!(m.origin, "IMDb");
        assert_eq!(m.title, "The Matrix");
        assert_eq!(m.year, Some(1999));
        assert_eq!(m.rating, Some(8.7));
        assert_eq!(m.votes, Some(2_100_000));
        assert_eq!(m.runtime_min, Some(136));
        assert_eq!(m.certificate.as_deref(), Some("R"));
        assert_eq!(m.genres, ["Action", "Sci-Fi"]);
        assert_eq!(m.directors, ["Lana Wachowski", "Lilly Wachowski"]);
        assert_eq!(m.cast, ["Keanu Reeves", "Laurence Fishburne"]);
        assert!(m.poster.unwrap().ends_with("._V1_SX600.jpg"));
    }

    #[test]
    fn series_uses_creators_and_tolerates_nulls() {
        let m = parse(include_str!(
            "../../../tests/fixtures/meta/imdb_details_series.json"
        ))
        .unwrap();
        assert_eq!(m.title, "Breaking Bad");
        assert_eq!(m.directors, ["Vince Gilligan"]);
        assert_eq!(m.cast, ["Bryan Cranston"]);
        assert_eq!(m.runtime_min, Some(48));
        assert_eq!(m.plot, None);
        assert_eq!(m.certificate, None);
    }

    #[test]
    fn missing_title_is_an_error() {
        assert!(parse(r#"{"data":{"title":null}}"#).is_err());
        assert!(parse(r#"{"errors":[{"message":"boom"}]}"#).is_err());
    }

    #[test]
    fn id_validation() {
        assert!(valid_id("tt0133093"));
        for bad in ["tt", "0133093", "tt1\") { x", "nm0000206", ""] {
            assert!(!valid_id(bad), "{bad}");
        }
        let client = reqwest::blocking::Client::new();
        assert!(fetch(&client, "tt1\"}").is_err());
    }

    #[test]
    #[ignore = "network"]
    fn live_matrix() {
        let client = crate::meta::client().unwrap();
        let cands = crate::meta::imdb::suggest::search(&client, "The Matrix").unwrap();
        let want = crate::meta::title::clean("The Matrix (1999)");
        let c = crate::meta::pick::pick(&cands, &want, crate::model::MediaKind::Movie).unwrap();
        println!("picked {c:?}");
        let m = fetch(&client, &c.id).unwrap();
        println!("{m:#?}");
        assert_eq!(m.title, "The Matrix");
    }
}
