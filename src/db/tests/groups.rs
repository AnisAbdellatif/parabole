use super::*;

fn setup_two() -> (Db, ProfileId, SourceId) {
    let (mut db, p, s) = setup();
    db.replace_source_data(
        s,
        &playlist(
            &["a"],
            &[("1", "a", "One"), ("2", "a", "Two"), ("3", "a", "Three")],
        ),
    )
    .unwrap();
    (db, p, s)
}

#[test]
fn groups_are_ordered_and_items_append_without_duplicates() {
    let (db, p, s) = setup_two();
    let m = MediaKind::Live;
    let g1 = db.create_group(p, m, "News").unwrap();
    let g2 = db.create_group(p, m, "Kids").unwrap();
    assert!(db.groups(p, MediaKind::Movie).unwrap().is_empty());
    db.reorder_groups(&[g2, g1]).unwrap();
    let names: Vec<_> = db
        .groups(p, m)
        .unwrap()
        .into_iter()
        .map(|g| g.name)
        .collect();
    assert_eq!(names, ["Kids", "News"]);
    for k in ["2", "1", "2"] {
        db.add_to_group(g1, s, k).unwrap();
    }
    let keys = |db: &Db| -> Vec<String> {
        db.group_streams(p, g1, m)
            .unwrap()
            .into_iter()
            .map(|r| r.key)
            .collect()
    };
    assert_eq!(keys(&db), ["2", "1"]);
    assert_eq!(db.groups(p, m).unwrap()[1].count, 2);
    db.reorder_group_items(g1, &[(s, "1".into()), (s, "2".into())])
        .unwrap();
    assert_eq!(keys(&db), ["1", "2"]);
    db.remove_from_group(g1, s, "1").unwrap();
    assert_eq!(keys(&db), ["2"]);
    db.rename_group(g1, "Daily").unwrap();
    db.delete_group(g1).unwrap();
    assert_eq!(db.groups(p, m).unwrap().len(), 1);
}

#[test]
fn group_items_survive_refresh_and_show_missing() {
    let (mut db, p, s) = setup_two();
    let m = MediaKind::Live;
    let g = db.create_group(p, m, "G").unwrap();
    db.add_to_group(g, s, "1").unwrap();
    db.add_to_group(g, s, "2").unwrap();
    db.replace_source_data(s, &playlist(&["a"], &[("2", "a", "Two renamed")]))
        .unwrap();
    let rows = db.group_streams(p, g, m).unwrap();
    assert_eq!((rows[0].name.as_str(), rows[0].missing), ("One", true));
    assert_eq!(
        (rows[1].name.as_str(), rows[1].missing),
        ("Two renamed", false)
    );
    db.set_favorite(p, s, m, "2", true).unwrap();
    assert!(db.group_streams(p, g, m).unwrap()[1].favorite);
}

#[test]
fn groups_are_per_profile_and_cascade() {
    let (db, p, s) = setup_two();
    let m = MediaKind::Live;
    let other = db.create_profile("Other").unwrap();
    let g = db.create_group(p, m, "G").unwrap();
    db.add_to_group(g, s, "1").unwrap();
    assert!(db.groups(other, m).unwrap().is_empty());
    assert!(db.group_streams(other, g, m).unwrap().is_empty());
    db.delete_profile(p).unwrap();
    let items: i64 = db
        .conn
        .query_row("SELECT COUNT(*) FROM stream_group_items", [], |r| r.get(0))
        .unwrap();
    assert_eq!(items, 0);
}
