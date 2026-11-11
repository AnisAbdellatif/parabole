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
    assert_eq!(db.search(p, None, m, "can cinema", 10).unwrap().len(), 1);
    assert_eq!(db.search(p, None, m, "canal", 10).unwrap().len(), 2);
    assert_eq!(db.search(p, None, m, "canal", 1).unwrap().len(), 1);
    assert!(
        db.search(p, None, MediaKind::Movie, "canal", 10)
            .unwrap()
            .is_empty()
    );
    assert!(db.search(p, None, m, "   ", 10).unwrap().is_empty());
    db.set_stream_hidden(p, s, m, "2", true).unwrap();
    assert_eq!(db.search(p, Some(s), m, "canal", 10).unwrap().len(), 1);
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
        assert!(db.search(p, None, MediaKind::Live, q, 10).is_ok(), "{q}");
    }
    assert_eq!(
        db.search(p, None, MediaKind::Live, "foo OR", 10)
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        db.search(p, None, MediaKind::Live, "foo*", 10)
            .unwrap()
            .len(),
        1
    );
}
