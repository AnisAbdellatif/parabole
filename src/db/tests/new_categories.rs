use super::*;

#[test]
fn new_categories_hidden_only_for_opted_in_profile() {
    let (mut db, p1, s) = setup();
    let p2 = db.create_profile("Kids").unwrap();
    db.link_source(p2, s, true).unwrap();
    db.replace_source_data(s, &playlist(&["a", "b"], &[("1", "a", "One")]))
        .unwrap();
    assert!(!db.hide_new_categories(p2, s).unwrap());
    db.set_hide_new_categories(p2, s, true).unwrap();
    assert!(db.hide_new_categories(p2, s).unwrap());
    db.replace_source_data(s, &playlist(&["a", "b", "c"], &[("1", "a", "One")]))
        .unwrap();
    assert_eq!(cat_keys(&db, p1, false), ["a", "b", "c"]);
    assert_eq!(cat_keys(&db, p2, false), ["a", "b"]);
    db.set_hide_new_categories(p2, s, false).unwrap();
    db.replace_source_data(s, &playlist(&["a", "b", "c", "d"], &[]))
        .unwrap();
    assert_eq!(cat_keys(&db, p2, false), ["a", "b", "d"]);
}

#[test]
fn first_import_hides_nothing() {
    let (mut db, p, s) = setup();
    db.set_hide_new_categories(p, s, true).unwrap();
    db.replace_source_data(s, &playlist(&["a"], &[])).unwrap();
    assert_eq!(cat_keys(&db, p, false), ["a"]);
}
