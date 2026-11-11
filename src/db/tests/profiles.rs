use super::*;

#[test]
fn profiles_have_isolated_overlays() {
    let (mut db, p1, s) = setup();
    let p2 = db.create_profile("Kids").unwrap();
    db.link_source(p2, s, true).unwrap();
    db.replace_source_data(
        s,
        &playlist(&["a", "b"], &[("1", "a", "One"), ("2", "b", "Two")]),
    )
    .unwrap();
    db.set_category_hidden(p2, s, MediaKind::Live, "b", true)
        .unwrap();
    db.set_favorite(p2, s, MediaKind::Live, "1", true).unwrap();
    assert_eq!(cat_keys(&db, p1, false), ["a", "b"]);
    assert_eq!(cat_keys(&db, p2, false), ["a"]);
    assert!(db.favorites(p1, MediaKind::Live).unwrap().is_empty());
    assert_eq!(
        db.search(p2, None, MediaKind::Live, "two", 10)
            .unwrap()
            .len(),
        0
    );
}

#[test]
fn unlinked_source_is_invisible_and_delete_profile_keeps_sources() {
    let (mut db, p1, s) = setup();
    db.replace_source_data(s, &playlist(&["a"], &[("1", "a", "One")]))
        .unwrap();
    let p2 = db.create_profile("Other").unwrap();
    assert!(cat_keys(&db, p2, false).is_empty());
    db.set_favorite(p1, s, MediaKind::Live, "1", true).unwrap();
    db.set_setting(Some(p1), "k", "v").unwrap();
    db.delete_profile(p1).unwrap();
    assert_eq!(db.source(s).unwrap().name, "A");
    assert_eq!(db.get_setting(Some(p1), "k").unwrap(), None);
    let n: i64 = db
        .conn
        .query_row("SELECT COUNT(*) FROM stream_prefs", [], |r| r.get(0))
        .unwrap();
    assert_eq!(n, 0);
}

#[test]
fn refresh_result_and_account_fields() {
    let (mut db, _, s) = setup();
    db.set_refresh_result(s, Some("boom")).unwrap();
    let src = db.source(s).unwrap();
    assert_eq!(
        (src.last_refresh, src.last_error.as_deref()),
        (None, Some("boom"))
    );
    let mut data = playlist(&[], &[]);
    data.epg_url = Some("http://epg".into());
    db.replace_source_data(s, &data).unwrap();
    db.set_refresh_result(s, None).unwrap();
    let src = db.source(s).unwrap();
    assert!(src.last_refresh.is_some() && src.last_error.is_none());
    let epg: String = db
        .conn
        .query_row("SELECT epg_url FROM sources", [], |r| r.get(0))
        .unwrap();
    assert_eq!(epg, "http://epg");
}
