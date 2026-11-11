use super::*;
use crate::model::{PlayInfo, SourceKind};
use serde_json::{Value, json};

macro_rules! fixture {
    ($name:literal) => {
        serde_json::from_str::<Value>(include_str!(concat!(
            "../../../tests/fixtures/sources/",
            $name
        )))
        .unwrap()
    };
}

fn list(v: Value) -> Vec<Value> {
    v.as_array().unwrap().clone()
}

fn playlist() -> ParsedPlaylist {
    let mut out = ParsedPlaylist::default();
    let cats = list(fixture!("categories.json"));
    convert::convert_media(
        MediaKind::Live,
        &cats,
        &list(fixture!("live_streams.json")),
        &mut out,
    );
    convert::convert_media(
        MediaKind::Movie,
        &cats,
        &list(fixture!("vod_streams.json")),
        &mut out,
    );
    convert::convert_media(
        MediaKind::Series,
        &cats,
        &list(fixture!("series.json")),
        &mut out,
    );
    out
}

fn stream<'a>(
    p: &'a ParsedPlaylist,
    media: MediaKind,
    key: &str,
) -> &'a crate::model::ParsedStream {
    p.streams
        .iter()
        .find(|s| s.media == media && s.ext_key == key)
        .unwrap()
}

#[test]
fn lenient_readers() {
    use de::*;
    assert_eq!(opt_string(Some(&json!(12))).as_deref(), Some("12"));
    for v in [json!(""), json!(false), Value::Null, json!([])] {
        assert_eq!(opt_string(Some(&v)), None);
    }
    assert_eq!(opt_i64(Some(&json!("7"))), Some(7));
    assert_eq!(opt_i64(Some(&json!(7.0))), Some(7));
    assert_eq!(opt_i64(Some(&json!(""))), None);
    assert_eq!(opt_i64(Some(&Value::Null)), None);
    assert!(
        [json!(1), json!("1"), json!(true)]
            .iter()
            .all(|v| bool01(Some(v)))
    );
    assert!(
        [json!(0), json!("0"), json!(false), Value::Null]
            .iter()
            .all(|v| !bool01(Some(v)))
    );
    assert_eq!(string_list(Some(&json!("a"))), ["a"]);
    assert_eq!(string_list(Some(&json!(["a", "b"]))), ["a", "b"]);
}

#[test]
fn live_conversion_with_type_chaos() {
    let p = playlist();
    let bbc = stream(&p, MediaKind::Live, "101");
    assert_eq!((bbc.num, bbc.archive_days), (Some(1), Some(3)));
    assert_eq!(bbc.epg_id.as_deref(), Some("bbc.uk"));
    assert_eq!(bbc.logo.as_deref(), Some("http://i/b.png"));
    let espn = stream(&p, MediaKind::Live, "102");
    assert_eq!(
        (espn.logo.clone(), espn.epg_id.clone(), espn.archive_days),
        (None, None, None)
    );
    assert_eq!(espn.category_key.as_deref(), Some("2"));
    let live: Vec<_> = p
        .streams
        .iter()
        .filter(|s| s.media == MediaKind::Live)
        .collect();
    assert_eq!(
        live.len(),
        4,
        "garbage, id-less and duplicate items are skipped"
    );
}

#[test]
fn unknown_categories_get_uncategorized() {
    let p = playlist();
    for key in ["103", "104"] {
        assert_eq!(
            stream(&p, MediaKind::Live, key).category_key.as_deref(),
            Some("uncategorized")
        );
    }
    let live_cats: Vec<_> = p
        .categories
        .iter()
        .filter(|c| c.media == MediaKind::Live)
        .map(|c| c.name.as_str())
        .collect();
    assert_eq!(live_cats, ["News", "Sports", "Uncategorized"]);
}

#[test]
fn vod_and_series_conversion() {
    let p = playlist();
    let film = stream(&p, MediaKind::Movie, "5");
    assert_eq!(film.container_ext.as_deref(), Some("mkv"));
    let extra: Value = serde_json::from_str(film.extra_json.as_deref().unwrap()).unwrap();
    assert_eq!(extra, json!({"rating": "7.5", "added": "1700000000"}));
    assert!(stream(&p, MediaKind::Movie, "6").extra_json.is_none());
    let show = stream(&p, MediaKind::Series, "9");
    assert_eq!(show.logo.as_deref(), Some("http://i/s.png"));
    assert!(
        show.extra_json
            .as_deref()
            .unwrap()
            .contains("http://i/2.jpg")
    );
    let show2 = stream(&p, MediaKind::Series, "10");
    assert_eq!(show2.logo, None);
    assert!(
        show2
            .extra_json
            .as_deref()
            .unwrap()
            .contains("http://i/3.jpg")
    );
    assert!(stream(&p, MediaKind::Series, "11").extra_json.is_none());
}

#[test]
fn account_checks() {
    assert!(convert::check_account(&fixture!("account_ok.json")).is_ok());
    let err = convert::check_account(&fixture!("account_bad.json")).unwrap_err();
    assert_eq!(err.to_string(), "invalid credentials");
    assert!(convert::check_account(&json!({})).is_err());
    let json = convert::account_json(&fixture!("account_ok.json"));
    assert!(json.contains("user_info") && json.contains("server_info"));
}

#[test]
fn series_info_object_keyed() {
    let seasons = series::parse_seasons(&fixture!("series_info_object.json"));
    let shape: Vec<_> = seasons
        .iter()
        .map(|s| (s.number, s.name.as_str(), s.episodes.len()))
        .collect();
    assert_eq!(
        shape,
        [(1, "First", 2), (2, "Season 2", 2)],
        "season 3 has no episodes"
    );
    let ids: Vec<_> = seasons[1].episodes.iter().map(|e| e.id.as_str()).collect();
    assert_eq!(ids, ["21", "22"]);
    assert_eq!(seasons[1].episodes[1].container_ext.as_deref(), Some("mkv"));
    let first = &seasons[0].episodes;
    assert_eq!(
        (first[0].id.as_str(), first[0].title.as_str()),
        ("12", "A2")
    );
    assert_eq!(
        (first[1].id.as_str(), first[1].title.as_str()),
        ("13", "Episode 0")
    );
}

#[test]
fn series_info_array_and_empty() {
    let seasons = series::parse_seasons(&fixture!("series_info_array.json"));
    assert_eq!(
        seasons
            .iter()
            .map(|s| (s.number, s.episodes.len()))
            .collect::<Vec<_>>(),
        [(1, 1), (2, 1)]
    );
    assert!(series::parse_seasons(&fixture!("series_info_empty.json")).is_empty());
}

fn source(url: &str, user: &str, pass: &str) -> Source {
    Source {
        id: 1,
        name: "s".into(),
        kind: SourceKind::Xtream,
        url: url.into(),
        username: Some(user.into()),
        password: Some(pass.into()),
        user_agent: None,
        mirrors: Vec::new(),
        output_format: None,
        last_refresh: None,
        last_error: None,
    }
}

fn info(media: MediaKind, ext: Option<&str>) -> PlayInfo {
    PlayInfo {
        source_id: 1,
        media,
        key: "42".into(),
        name: "n".into(),
        url: None,
        container_ext: ext.map(str::to_string),
        headers: Default::default(),
    }
}

#[test]
fn url_building() {
    let mut s = source("http://h.tv:8080", "us er", "p/a%ss@");
    let live = urls::resolve(&s, &info(MediaKind::Live, None)).unwrap();
    assert_eq!(live, "http://h.tv:8080/live/us%20er/p%2Fa%25ss@/42.ts");
    s.output_format = Some("m3u8".into());
    assert!(
        urls::resolve(&s, &info(MediaKind::Live, None))
            .unwrap()
            .ends_with("/42.m3u8")
    );
    assert!(
        urls::resolve(&s, &info(MediaKind::Movie, Some("mkv")))
            .unwrap()
            .ends_with("/movie/us%20er/p%2Fa%25ss@/42.mkv")
    );
    assert!(
        urls::resolve(&s, &info(MediaKind::Movie, None))
            .unwrap()
            .ends_with("/42.mp4")
    );
    assert_eq!(
        urls::resolve(&s, &info(MediaKind::Series, None))
            .unwrap_err()
            .to_string(),
        "select an episode"
    );
    assert_eq!(
        urls::episode(&s, "77", None).unwrap(),
        "http://h.tv:8080/series/us%20er/p%2Fa%25ss@/77.mp4"
    );
    let prefixed = source("http://h.tv/panel/", "u", "p");
    assert_eq!(
        urls::episode(&prefixed, "1", Some("mkv")).unwrap(),
        "http://h.tv/panel/series/u/p/1.mkv"
    );
}
