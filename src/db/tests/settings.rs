use super::*;

#[test]
fn settings_global_vs_profile() {
    let (db, p, _) = setup();
    assert_eq!(db.get_setting(None, "theme").unwrap(), None);
    db.set_setting(None, "theme", "\"dark\"").unwrap();
    db.set_setting(None, "theme", "\"light\"").unwrap();
    db.set_setting(Some(p), "theme", "\"blue\"").unwrap();
    assert_eq!(
        db.get_setting(None, "theme").unwrap().as_deref(),
        Some("\"light\"")
    );
    assert_eq!(
        db.get_setting(Some(p), "theme").unwrap().as_deref(),
        Some("\"blue\"")
    );
    let n: i64 = db
        .conn
        .query_row("SELECT COUNT(*) FROM settings", [], |r| r.get(0))
        .unwrap();
    assert_eq!(n, 2);
}
