use super::*;

#[test]
fn copy_appends_missing_favorites_in_order_without_duplicates() {
    let (mut db, p1, a) = setup();
    let b = db.add_source(p1, &source("B")).unwrap();
    let m = MediaKind::Live;
    db.replace_source_data(
        a,
        &playlist(&["x"], &[("1", "x", "One"), ("2", "x", "Two")]),
    )
    .unwrap();
    db.replace_source_data(b, &playlist(&["x"], &[("3", "x", "Three")]))
        .unwrap();
    let p2 = db.create_profile("P2").unwrap();
    db.link_source(p2, a, true).unwrap();
    for (s, k) in [(b, "3"), (a, "2"), (a, "1")] {
        db.set_favorite(p1, s, m, k, true).unwrap();
    }
    db.set_favorite(p2, a, m, "1", true).unwrap();
    assert_eq!(db.copy_favorites(p1, p2).unwrap(), 1);
    let keys: Vec<_> = db
        .favorites(p2, m)
        .unwrap()
        .into_iter()
        .map(|r| r.key)
        .collect();
    assert_eq!(keys, ["1", "2"]);
    assert_eq!(db.copy_favorites(p1, p2).unwrap(), 0);
    let all: Vec<_> = db
        .favorites(p1, m)
        .unwrap()
        .into_iter()
        .map(|r| (r.source_id, r.key))
        .collect();
    assert_eq!(
        all,
        [(b, "3".to_string()), (a, "2".into()), (a, "1".into())]
    );
}
