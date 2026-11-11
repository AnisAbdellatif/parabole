use super::*;
use crate::model::{MediaKind, ParsedStream};
use std::io::Cursor;

fn parse_bytes(b: &[u8]) -> ParsedPlaylist {
    parse(Cursor::new(b)).unwrap()
}

fn messy() -> ParsedPlaylist {
    parse_bytes(include_bytes!("../../../tests/fixtures/sources/messy.m3u"))
}

fn by_url<'a>(p: &'a ParsedPlaylist, tail: &str) -> &'a ParsedStream {
    p.streams
        .iter()
        .find(|s| s.url.as_deref().unwrap().contains(tail))
        .unwrap()
}

#[test]
fn bom_crlf_header_and_counts() {
    let p = messy();
    assert_eq!(
        p.epg_url.as_deref(),
        Some("http://epg.example/guide.xml.gz")
    );
    assert_eq!(p.streams.len(), 9);
}

#[test]
fn commas_in_attrs_and_title() {
    let s = by_url(&messy(), "/1.ts").clone();
    assert_eq!(s.name, "BBC One, HD");
    assert_eq!(s.logo.as_deref(), Some("http://img/a,b.png"));
    assert_eq!(s.category_key.as_deref(), Some("UK"));
    assert_eq!(s.ext_key, "bbc.one");
}

#[test]
fn single_quotes_unquoted_and_vlcopt() {
    let s = by_url(&messy(), "/2.ts").clone();
    assert_eq!((s.name.as_str(), s.ext_key.as_str()), ("CNN", "cnn"));
    assert_eq!(s.category_key.as_deref(), Some("News"));
    assert_eq!((s.num, s.archive_days), (Some(7), Some(3)));
    assert_eq!(s.headers.user_agent.as_deref(), Some("Mozilla/5.0"));
    assert_eq!(s.headers.referrer.as_deref(), Some("http://ref.example/"));
}

#[test]
fn extgrp_missing_group_and_duplicate_names() {
    let p = messy();
    assert_eq!(by_url(&p, "/3.ts").category_key.as_deref(), Some("Sports"));
    let (a, b) = (by_url(&p, "/4.ts"), by_url(&p, "/5.ts"));
    assert_eq!(a.category_key.as_deref(), Some("Uncategorized"));
    assert_eq!(a.ext_key, "Dup\u{1f}Uncategorized");
    assert_eq!(b.ext_key, "Dup\u{1f}Uncategorized#2");
}

#[test]
fn pipe_headers_stripped_from_url() {
    let s = by_url(&messy(), "/5.ts").clone();
    assert_eq!(s.url.as_deref(), Some("http://h/live/u/p/5.ts"));
    assert_eq!(s.headers.user_agent.as_deref(), Some("Foo"));
    assert_eq!(s.headers.referrer.as_deref(), Some("http://bar/"));
}

#[test]
fn media_classification_and_drm_and_name_fallback() {
    let p = messy();
    assert_eq!(by_url(&p, "/6.mkv").media, MediaKind::Movie);
    assert_eq!(by_url(&p, "/7.mp4").media, MediaKind::Series);
    assert_eq!(by_url(&p, "/9.avi").media, MediaKind::Movie);
    assert_eq!(by_url(&p, "/1.ts").media, MediaKind::Live);
    let drm = by_url(&p, "/8.mpd");
    assert_eq!(drm.name, "From Attr");
    assert_eq!(drm.extra_json.as_deref(), Some(r#"{"drm":true}"#));
    assert!(by_url(&p, "/1.ts").extra_json.is_none());
}

#[test]
fn categories_per_media_in_first_appearance_order() {
    let p = messy();
    let keys: Vec<_> = p
        .categories
        .iter()
        .map(|c| (c.media, c.ext_key.as_str()))
        .collect();
    use MediaKind::*;
    assert_eq!(
        keys,
        [
            (Live, "UK"),
            (Live, "News"),
            (Live, "Sports"),
            (Live, "Uncategorized"),
            (Movie, "Movies"),
            (Series, "Shows"),
            (Live, "Locked"),
            (Movie, "Uncategorized")
        ]
    );
}

#[test]
fn duplicate_tvg_id_falls_back_to_name_key() {
    let p = parse_bytes(b"#EXTINF:-1 tvg-id=\"x\" group-title=\"G\",A\nhttp://h/1\n#EXTINF:-1 tvg-id=\"x\" group-title=\"G\",B\nhttp://h/2\n");
    assert_eq!(p.streams[0].ext_key, "A\u{1f}G");
    assert_eq!(p.streams[1].ext_key, "B\u{1f}G");
}

#[test]
fn invalid_utf8_is_lossy() {
    let p = parse_bytes(b"#EXTINF:-1,Caf\xe9\nhttp://h/1\n");
    assert_eq!(p.streams[0].name, "Caf\u{fffd}");
}

#[test]
fn hls_detection() {
    let is_hls = |b: &[u8]| parse_detect(Cursor::new(b)).unwrap().1;
    assert!(is_hls(include_bytes!(
        "../../../tests/fixtures/sources/hls_master.m3u8"
    )));
    assert!(is_hls(include_bytes!(
        "../../../tests/fixtures/sources/hls_media.m3u8"
    )));
    assert!(!is_hls(include_bytes!(
        "../../../tests/fixtures/sources/messy.m3u"
    )));
}

#[test]
fn hundred_thousand_entries() {
    let mut text = String::from("#EXTM3U\n");
    for i in 0..100_000 {
        text.push_str(&format!(
            "#EXTINF:-1 tvg-id=\"id{i}\" tvg-logo=\"http://l/{i}.png\" group-title=\"G{}\",Channel {i}\nhttp://h/live/{i}.ts\n",
            i % 50
        ));
    }
    let p = parse_bytes(text.as_bytes());
    assert_eq!(p.streams.len(), 100_000);
    assert_eq!(p.categories.len(), 50);
}
