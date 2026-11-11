use super::*;

#[test]
fn search_prefix_diacritics_and_filters() {
    let (mut db, p, s) = setup();
    let m = MediaKind::Live;
    db.replace_source_data(
        s,
        &playlist(
            &["a", "h"],
            &[
                ("1", "a", "Canal Plus Cinéma"),
                ("2", "a", "Canal Sport"),
                ("3", "h", "Canal Hidden"),
            ],
        ),
    )
    .unwrap();
    db.set_category_hidden(p, s, m, "h", true).unwrap();
    assert_eq!(
        db.search(p, None, Some(m), "can cinema", 10).unwrap().len(),
        1
    );
    assert_eq!(db.search(p, None, Some(m), "canal", 10).unwrap().len(), 2);
    assert_eq!(db.search(p, None, Some(m), "canal", 1).unwrap().len(), 1);
    assert!(
        db.search(p, None, Some(MediaKind::Movie), "canal", 10)
            .unwrap()
            .is_empty()
    );
    assert!(db.search(p, None, Some(m), "   ", 10).unwrap().is_empty());
    db.set_stream_hidden(p, s, m, "2", true).unwrap();
    assert_eq!(
        db.search(p, Some(s), Some(m), "canal", 10).unwrap().len(),
        1
    );
}

#[test]
fn search_input_is_sanitized() {
    let (mut db, p, s) = setup();
    db.replace_source_data(s, &playlist(&["a"], &[("1", "a", "Foo Bar")]))
        .unwrap();
    for q in [
        "\"",
        "foo\"",
        "*",
        "foo*",
        "-foo",
        "foo OR",
        "OR",
        "NOT foo",
        "foo AND",
        "(",
        "a:b",
        "^foo",
        "NEAR(foo bar)",
    ] {
        assert!(
            db.search(p, None, Some(MediaKind::Live), q, 10).is_ok(),
            "{q}"
        );
    }
    assert_eq!(
        db.search(p, None, Some(MediaKind::Live), "foo OR", 10)
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        db.search(p, None, Some(MediaKind::Live), "foo*", 10)
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn search_filters_by_media_kind_and_source() {
    let (mut db, p, a) = setup();
    let b = db.add_source(p, &source("B")).unwrap();
    db.replace_source_data(a, &playlist(&["x"], &[("1", "x", "Alpha Live")]))
        .unwrap();
    let mut data = playlist(
        &["x"],
        &[("2", "x", "Alpha Film"), ("3", "x", "Alpha Show")],
    );
    data.streams[0].media = MediaKind::Movie;
    data.streams[1].media = MediaKind::Series;
    db.replace_source_data(b, &data).unwrap();
    let n = |source, media| db.search(p, source, media, "alpha", 10).unwrap().len();
    assert_eq!(n(None, None), 3);
    assert_eq!(n(None, Some(MediaKind::Movie)), 1);
    assert_eq!(n(Some(a), None), 1);
    assert_eq!(n(Some(b), None), 2);
    assert_eq!(n(Some(a), Some(MediaKind::Movie)), 0);
    let kinds: Vec<_> = db
        .search(p, None, None, "alpha", 10)
        .unwrap()
        .iter()
        .map(|r| r.media)
        .collect();
    assert_eq!(kinds, MediaKind::ALL);
}
