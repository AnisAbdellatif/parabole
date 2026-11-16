use super::*;

fn id(db: &Db, p: ProfileId, name: &str) -> i64 {
    let rows = db.live_variants(p, None).unwrap();
    rows.into_iter().find(|(r, _)| r.name == name).unwrap().0.id
}

fn seeded() -> (Db, ProfileId, SourceId) {
    let (mut db, p, s) = setup();
    let data = playlist(
        &["a", "b", "h"],
        &[
            ("k1", "a", "One"),
            ("k2", "a", "Two"),
            ("k3", "b", "Three"),
            ("k4", "h", "Four"),
        ],
    );
    db.replace_source_data(s, &data).unwrap();
    db.set_category_hidden(p, s, MediaKind::Live, "h", true)
        .unwrap();
    db.set_stream_hidden(p, s, MediaKind::Live, "k2", true)
        .unwrap();
    (db, p, s)
}

#[test]
fn default_selection_is_visible_categories() {
    let (mut db, p, s) = seeded();
    assert!(db.scan_categories(p).unwrap().is_empty());
    let sel = db.scan_selection_or_default(p).unwrap();
    assert_eq!(sel, [(s, "a".to_string()), (s, "b".to_string())]);
    let q = db.scan_queue(p, 20).unwrap();
    assert_eq!(q.len(), 2);
    assert_eq!(db.scan_counts(p).unwrap(), (0, 2));
    db.set_scan_categories(p, &[]).unwrap();
    assert!(db.scan_selection_or_default(p).unwrap().is_empty());
    assert!(db.scan_queue(p, 20).unwrap().is_empty());
    assert_eq!(db.scan_counts(p).unwrap(), (0, 0));
}

#[test]
fn queue_follows_selection_and_skips_probed() {
    let (mut db, p, s) = seeded();
    // The hidden category can still be chosen explicitly.
    db.set_scan_categories(p, &[(s, "a".into()), (s, "h".into())])
        .unwrap();
    assert_eq!(db.scan_categories(p).unwrap().len(), 2);
    assert_eq!(db.scan_queue(p, 20).unwrap().len(), 2);
    assert_eq!(db.scan_queue(p, 1).unwrap(), [id(&db, p, "One")]);
    db.save_quality(s, "k1", &Quality::default()).unwrap();
    db.record_failure(s, "k4").unwrap();
    assert!(db.scan_queue(p, 20).unwrap().is_empty());
    assert_eq!(db.scan_counts(p).unwrap(), (2, 2));
}

#[test]
fn rescan_epoch_requeues_everything() {
    let (db, p, s) = seeded();
    db.save_quality(s, "k1", &Quality::default()).unwrap();
    assert_eq!(db.scan_counts(p).unwrap(), (1, 2));
    assert_eq!(db.scan_since(p).unwrap(), 0);
    let future = 4_000_000_000;
    db.set_scan_since(p, future).unwrap();
    assert_eq!(db.scan_since(p).unwrap(), future);
    assert_eq!(db.scan_queue(p, 20).unwrap().len(), 2);
    assert_eq!(db.scan_counts(p).unwrap(), (0, 2));
}

#[test]
fn other_profiles_sources_are_not_scanned() {
    let (mut db, p, _) = seeded();
    let other = db.create_profile("Other").unwrap();
    let b = db.add_source(other, &source("B")).unwrap();
    db.replace_source_data(b, &playlist(&["a"], &[("x", "a", "X")]))
        .unwrap();
    assert_eq!(db.scan_counts(p).unwrap(), (0, 2));
    assert_eq!(db.scan_counts(other).unwrap(), (0, 1));
}
