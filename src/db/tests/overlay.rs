use super::*;

fn names(rows: &[StreamRow]) -> Vec<String> {
    rows.iter().map(|r| r.key.clone()).collect()
}

#[test]
fn overlay_survives_reimport() {
    let (mut db, p, s) = setup();
    let m = MediaKind::Live;
    db.replace_source_data(
        s,
        &playlist(
            &["a", "b", "c", "d"],
            &[
                ("1", "a", "One"),
                ("2", "a", "Two"),
                ("3", "b", "Three"),
                ("4", "c", "Four"),
            ],
        ),
    )
    .unwrap();
    assert_eq!(cat_keys(&db, p, false), ["a", "b", "c", "d"]);

    db.set_category_hidden(p, s, m, "d", true).unwrap();
    db.set_category_pinned(p, s, m, "c", true).unwrap();
    db.reorder_categories(p, s, m, &["b".into(), "a".into(), "c".into()])
        .unwrap();
    db.set_favorite(p, s, m, "4", true).unwrap();
    db.set_favorite(p, s, m, "1", true).unwrap();
    db.set_stream_hidden(p, s, m, "2", true).unwrap();
    assert_eq!(cat_keys(&db, p, false), ["c", "b", "a"]);
    assert_eq!(cat_keys(&db, p, true), ["c", "b", "a", "d"]);
    assert_eq!(names(&db.favorites(p, m).unwrap()), ["4", "1"]);
    let counts: Vec<i64> = db
        .categories(p, None, m, false)
        .unwrap()
        .iter()
        .map(|c| c.count)
        .collect();
    assert_eq!(counts, [1, 1, 1]);

    // shuffled, "4" removed, new category "z" and stream "9"
    db.replace_source_data(
        s,
        &playlist(
            &["z", "d", "a", "c", "b"],
            &[
                ("9", "z", "Nine"),
                ("3", "b", "Three"),
                ("2", "a", "Two"),
                ("1", "a", "One"),
            ],
        ),
    )
    .unwrap();
    assert_eq!(cat_keys(&db, p, false), ["c", "b", "a", "z"]);
    let cats = db.categories(p, None, m, true).unwrap();
    assert!(cats.iter().find(|c| c.key == "d").unwrap().hidden);
    assert!(cats.iter().find(|c| c.key == "c").unwrap().pinned);
    let favs = db.favorites(p, m).unwrap();
    assert_eq!(names(&favs), ["4", "1"]);
    assert!(favs[0].missing && !favs[1].missing);
    let in_a = db.streams_in_category(p, s, m, "a", false).unwrap();
    assert_eq!(names(&in_a), ["1"]);
    let in_a = db.streams_in_category(p, s, m, "a", true).unwrap();
    assert_eq!(names(&in_a), ["2", "1"]);
    assert!(in_a[0].hidden && in_a[1].favorite);

    db.replace_source_data(
        s,
        &playlist(&["c"], &[("4", "c", "Four"), ("1", "c", "One")]),
    )
    .unwrap();
    assert_eq!(names(&db.favorites(p, m).unwrap()), ["4", "1"]);
    assert!(db.favorites(p, m).unwrap().iter().all(|f| !f.missing));

    // "4" vanishes: favorite stays in order, flagged missing, with its last name
    db.replace_source_data(s, &playlist(&["c"], &[("1", "c", "One")]))
        .unwrap();
    let favs = db.favorites(p, m).unwrap();
    assert_eq!(names(&favs), ["4", "1"]);
    assert!(favs[0].missing && !favs[1].missing);
    assert_eq!(favs[0].name, "Four");
    db.set_favorite(p, s, m, "4", false).unwrap();
    assert_eq!(names(&db.favorites(p, m).unwrap()), ["1"]);
}

#[test]
fn failed_import_rolls_back() {
    let (mut db, p, s) = setup();
    db.replace_source_data(s, &playlist(&["a"], &[("1", "a", "One")]))
        .unwrap();
    let mut bad = playlist(&["b"], &[("2", "b", "Two")]);
    bad.streams[0].extra_json = None;
    db.conn
        .execute_batch(
            "CREATE TRIGGER boom BEFORE INSERT ON streams BEGIN SELECT RAISE(ABORT, 'x'); END;",
        )
        .unwrap();
    assert!(db.replace_source_data(s, &bad).is_err());
    db.conn.execute_batch("DROP TRIGGER boom").unwrap();
    assert_eq!(cat_keys(&db, p, false), ["a"]);
    assert_eq!(
        db.streams_in_category(p, s, MediaKind::Live, "a", false)
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn reorder_favorites_pin_reset_and_recent() {
    let (mut db, p, s) = setup();
    let m = MediaKind::Live;
    db.replace_source_data(
        s,
        &playlist(
            &["a", "b"],
            &[("1", "a", "One"), ("2", "a", "Two"), ("3", "b", "Three")],
        ),
    )
    .unwrap();
    for k in ["1", "2", "3"] {
        db.set_favorite(p, s, m, k, true).unwrap();
    }
    db.set_favorite(p, s, m, "2", false).unwrap();
    db.reorder_favorites(p, m, &[(s, "3".into()), (s, "1".into())])
        .unwrap();
    assert_eq!(names(&db.favorites(p, m).unwrap()), ["3", "1"]);

    db.reorder_categories(p, s, m, &["b".into(), "a".into()])
        .unwrap();
    assert_eq!(cat_keys(&db, p, false), ["b", "a"]);
    db.reset_category_order(p, s, m).unwrap();
    assert_eq!(cat_keys(&db, p, false), ["a", "b"]);
    db.set_category_pinned(p, s, m, "b", true).unwrap();
    db.set_category_pinned(p, s, m, "b", false).unwrap();
    assert_eq!(cat_keys(&db, p, false), ["a", "b"]);

    db.record_watch(p, s, m, "1").unwrap();
    db.record_watch(p, s, m, "2").unwrap();
    db.conn
        .execute(
            "UPDATE history SET watched_at = 1 WHERE stream_key = '1'",
            [],
        )
        .unwrap();
    assert_eq!(names(&db.recent(p, m, 10).unwrap()), ["2", "1"]);
    let id = db.recent(p, m, 1).unwrap()[0].id;
    assert_eq!(db.play_info(id).unwrap().key, "2");
}

#[test]
fn delete_source_cascades_overlay_and_search() {
    let (mut db, p, s) = setup();
    let m = MediaKind::Live;
    db.replace_source_data(s, &playlist(&["a"], &[("1", "a", "One")]))
        .unwrap();
    db.set_favorite(p, s, m, "1", true).unwrap();
    db.set_category_hidden(p, s, m, "a", true).unwrap();
    db.record_watch(p, s, m, "1").unwrap();
    db.delete_source(s).unwrap();
    for t in [
        "categories",
        "streams",
        "category_prefs",
        "stream_prefs",
        "history",
        "profile_sources",
    ] {
        let n: i64 = db
            .conn
            .query_row(&format!("SELECT COUNT(*) FROM {t}"), [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 0, "{t}");
    }
    assert!(db.search(p, None, Some(m), "one", 10).unwrap().is_empty());
}

#[test]
fn bulk_hide_applies_all_in_one_go() {
    let (mut db, p, s) = setup();
    let m = MediaKind::Live;
    db.replace_source_data(s, &playlist(&["a", "b"], &[("1", "a", "One")]))
        .unwrap();
    db.set_categories_hidden(p, m, &[(s, "a".into(), true), (s, "b".into(), true)])
        .unwrap();
    assert!(cat_keys(&db, p, false).is_empty());
    assert_eq!(cat_keys(&db, p, true), ["a", "b"]);
}
