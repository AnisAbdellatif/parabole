use super::*;

#[test]
fn account_json_roundtrip() {
    let (db, p, _) = setup();
    let id = db.add_source(p, &source("acct")).unwrap();
    assert_eq!(db.account_json(id).unwrap(), None);
    db.set_account_json(id, r#"{"user_info":{}}"#).unwrap();
    assert_eq!(
        db.account_json(id).unwrap().as_deref(),
        Some(r#"{"user_info":{}}"#)
    );
}
