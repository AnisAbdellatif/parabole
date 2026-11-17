use super::*;

fn integrity(db: &Db) {
    db.conn
        .execute(
            "INSERT INTO streams_fts(streams_fts) VALUES ('integrity-check')",
            [],
        )
        .unwrap();
}

fn ids(db: &Db, p: ProfileId, q: &str) -> Vec<i64> {
    let mut v: Vec<i64> = db
        .search(p, None, None, q, 50)
        .unwrap()
        .iter()
        .map(|r| r.id)
        .collect();
    v.sort();
    v
}

fn stream_id(db: &Db, s: SourceId, key: &str) -> i64 {
    db.conn
        .query_row(
            "SELECT id FROM streams WHERE source_id = ?1 AND ext_key = ?2",
            (s, key),
            |r| r.get(0),
        )
        .unwrap()
}

#[test]
fn refresh_keeps_ids_and_reindexes_changed_names() {
    let (mut db, p, s) = setup();
    db.replace_source_data(
        s,
        &playlist(&["a"], &[("1", "a", "Alpha"), ("2", "a", "Beta")]),
    )
    .unwrap();
    let (id1, id2) = (stream_id(&db, s, "1"), stream_id(&db, s, "2"));
    integrity(&db);
    db.replace_source_data(
        s,
        &playlist(&["a"], &[("1", "a", "Gamma"), ("2", "a", "Beta")]),
    )
    .unwrap();
    assert_eq!((stream_id(&db, s, "1"), stream_id(&db, s, "2")), (id1, id2));
    assert!(ids(&db, p, "alpha").is_empty());
    assert_eq!(ids(&db, p, "gamma"), [id1]);
    assert_eq!(ids(&db, p, "beta"), [id2]);
    integrity(&db);
}

#[test]
fn removed_stream_leaves_table_and_search() {
    let (mut db, p, s) = setup();
    db.replace_source_data(
        s,
        &playlist(&["a"], &[("1", "a", "Alpha"), ("2", "a", "Beta")]),
    )
    .unwrap();
    db.replace_source_data(s, &playlist(&["a"], &[("2", "a", "Beta")]))
        .unwrap();
    let n: i64 = db
        .conn
        .query_row("SELECT COUNT(*) FROM streams", [], |r| r.get(0))
        .unwrap();
    assert_eq!(n, 1);
    assert!(ids(&db, p, "alpha").is_empty());
    assert_eq!(ids(&db, p, "beta").len(), 1);
    integrity(&db);
}

#[test]
fn other_source_stays_searchable() {
    let (mut db, p, a) = setup();
    let b = db.add_source(p, &source("B")).unwrap();
    db.replace_source_data(a, &playlist(&["a"], &[("1", "a", "Alpha")]))
        .unwrap();
    db.replace_source_data(b, &playlist(&["a"], &[("1", "a", "Alpine")]))
        .unwrap();
    db.replace_source_data(a, &playlist(&["a"], &[("1", "a", "Alpha2")]))
        .unwrap();
    assert_eq!(ids(&db, p, "alp").len(), 2);
    integrity(&db);
    db.delete_source(a).unwrap();
    assert_eq!(ids(&db, p, "alp").len(), 1);
    assert_eq!(ids(&db, p, "alpine"), [stream_id(&db, b, "1")]);
    integrity(&db);
}

#[test]
fn duplicate_key_first_wins() {
    let (mut db, p, s) = setup();
    let pl = playlist(&["a"], &[("1", "a", "First"), ("1", "a", "Second")]);
    db.replace_source_data(s, &pl).unwrap();
    db.replace_source_data(s, &pl).unwrap();
    assert_eq!(ids(&db, p, "first").len(), 1);
    assert!(ids(&db, p, "second").is_empty());
    integrity(&db);
}
