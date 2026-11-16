use super::*;
use std::io::{Cursor, Read, Write};

const NOW: i64 = 1_767_268_800; // 2026-01-01 12:00:00 UTC

fn chans() -> HashSet<String> {
    ["one.example", "two.example"].map(String::from).into()
}

fn prog(channel: &str, start: i64) -> Programme {
    Programme {
        channel: channel.into(),
        start,
        stop: start + 60,
        title: "t".into(),
        descr: None,
    }
}

#[test]
fn parse_filters_and_unescapes() {
    let xml = include_bytes!("../../tests/fixtures/epg/small.xml");
    let got = parse::parse(&xml[..], &chans(), Window::around(NOW)).unwrap();
    let titles: Vec<_> = got.iter().map(|p| p.title.as_str()).collect();
    assert_eq!(titles, ["News", "Tom & Jerry", "Movie"]);
    assert_eq!(got[0].descr.as_deref(), Some("Daily news"));
    assert_eq!(got[0].start, NOW);
    assert_eq!(got[1].channel, "two.example");
    assert_eq!(got[1].start, NOW + 3600 - 3600);
    assert_eq!(got[2].descr.as_deref(), Some("A <b>big</b> film"));
}

#[test]
fn parse_rejects_malformed_xml() {
    let err = parse::parse(&b"<tv><programme></tv>"[..], &chans(), Window::around(NOW));
    assert!(format!("{:#}", err.unwrap_err()).contains("invalid guide XML"));
}

#[test]
fn decode_plain_and_gzip() {
    let mut plain = String::new();
    fetch::decode(Cursor::new(b"<tv/>".to_vec()))
        .unwrap()
        .read_to_string(&mut plain)
        .unwrap();
    assert_eq!(plain, "<tv/>");

    let mut enc = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    enc.write_all(b"<tv/>").unwrap();
    let mut out = String::new();
    fetch::decode(Cursor::new(enc.finish().unwrap()))
        .unwrap()
        .read_to_string(&mut out)
        .unwrap();
    assert_eq!(out, "<tv/>");
}

#[test]
fn merge_dedups_first_wins() {
    let mut a = prog("c", 1);
    a.title = "first".into();
    let got = merge(vec![
        vec![a, prog("c", 2)],
        vec![prog("c", 1), prog("d", 1)],
    ]);
    assert_eq!(got.len(), 3);
    assert_eq!(got[0].title, "first");
}

#[test]
fn fetch_all_edge_cases() {
    assert!(
        fetch_all(&[], None, &chans(), Window::around(NOW))
            .unwrap()
            .is_empty()
    );
    let err = fetch_all(&["ftp://x/y".into()], None, &chans(), Window::around(NOW)).unwrap_err();
    assert!(format!("{err:#}").contains("guide download failed"));
}
