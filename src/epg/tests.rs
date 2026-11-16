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
    let got = parse::parse(&xml[..], Window::around(NOW), &mut |_| chans())
        .unwrap()
        .programmes;
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
    let err = parse::parse(
        &b"<tv><programme></tv>"[..],
        Window::around(NOW),
        &mut |_| chans(),
    );
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
    let ch = |names: &[&str]| GuideChannel {
        id: "c".into(),
        names: names.iter().map(|s| s.to_string()).collect(),
    };
    let got = merge(vec![
        ParsedGuide {
            channels: vec![ch(&["A"])],
            programmes: vec![a, prog("c", 2)],
        },
        ParsedGuide {
            channels: vec![ch(&["A", "B"])],
            programmes: vec![prog("c", 1), prog("d", 1)],
        },
    ]);
    assert_eq!(got.programmes.len(), 3);
    assert_eq!(got.programmes[0].title, "first");
    assert_eq!(got.channels.len(), 1);
    assert_eq!(got.channels[0].names, ["A", "B"]);
}

#[test]
fn fetch_all_edge_cases() {
    assert!(
        fetch_all(&[], None, Window::around(NOW), &mut |_| chans())
            .unwrap()
            .programmes
            .is_empty()
    );
    let err = fetch_all(
        &["ftp://x/y".into()],
        None,
        Window::around(NOW),
        &mut |_| chans(),
    )
    .unwrap_err();
    assert!(format!("{err:#}").contains("guide download failed"));
}

#[test]
fn parse_lists_channels_and_selects() {
    let xml = r#"<tv>
<channel id=" One.Example "><display-name>One &amp; Co</display-name><display-name><![CDATA[Uno]]></display-name><display-name>Uno</display-name><display-name> </display-name><icon src="x"/></channel>
<channel id="two.example"><display-name>Two</display-name></channel>
<programme start="20260101120000 +0000" stop="20260101130000 +0000" channel="one.example"><title>A</title></programme>
<programme start="20260101120000 +0000" stop="20260101130000 +0000" channel="two.example"><title>B</title></programme>
<channel id="late"><display-name>Late</display-name></channel>
</tv>"#;
    let mut seen = Vec::new();
    let got = parse::parse(xml.as_bytes(), Window::around(NOW), &mut |c| {
        seen.push(c.len());
        ["two.example".to_string()].into()
    })
    .unwrap();
    assert_eq!(seen, [2]);
    let ids: Vec<_> = got.channels.iter().map(|c| c.id.as_str()).collect();
    assert_eq!(ids, ["one.example", "two.example", "late"]);
    assert_eq!(got.channels[0].names, ["One & Co", "Uno"]);
    assert_eq!(got.programmes.len(), 1);
    assert_eq!(got.programmes[0].title, "B");
}

#[test]
fn parse_calls_select_at_eof_without_programmes() {
    let mut n = 0;
    let got = parse::parse(
        &b"<tv><channel id=\"a\"><display-name>A</display-name></channel></tv>"[..],
        Window::around(NOW),
        &mut |c| {
            n = c.len();
            HashSet::new()
        },
    )
    .unwrap();
    assert_eq!((n, got.channels.len()), (1, 1));
}
