use super::*;

fn with_epg(key: &str, name: &str, epg: Option<&str>) -> ParsedStream {
    let mut s = stream(key, "c", name);
    s.epg_id = epg.map(Into::into);
    s
}

fn load(db: &mut Db, s: SourceId, streams: Vec<ParsedStream>) {
    let mut data = playlist(&["c"], &[]);
    data.streams = streams;
    db.replace_source_data(s, &data).unwrap();
}

fn id_of(db: &Db, s: SourceId, key: &str) -> i64 {
    db.conn
        .query_row(
            "SELECT id FROM streams WHERE source_id = ?1 AND ext_key = ?2",
            (s, key),
            |r| r.get(0),
        )
        .unwrap()
}

fn names(db: &Db, p: ProfileId, id: i64) -> Vec<String> {
    db.alternates(p, id)
        .unwrap()
        .into_iter()
        .map(|a| a.source_name)
        .collect()
}

#[test]
fn matches_by_epg_id_then_normalized_name() {
    let (mut db, p, a) = setup();
    let b = db.add_source(p, &source("B")).unwrap();
    let c = db.add_source(p, &source("C")).unwrap();
    load(
        &mut db,
        a,
        vec![
            with_epg("1", "TF1 HD", Some("tf1.fr")),
            with_epg("2", "Arte", None),
        ],
    );
    load(
        &mut db,
        b,
        vec![
            with_epg("9", "Totally different", Some("tf1.fr")),
            with_epg("8", "Arte", None),
        ],
    );
    load(
        &mut db,
        c,
        vec![
            with_epg("7", "FR| TF1ᴴᴰ", None),
            with_epg("6", "Arté FHD", None),
            with_epg("5", "Arte Sport", None),
        ],
    );
    assert_eq!(names(&db, p, id_of(&db, a, "1")), ["B", "C"]);
    assert_eq!(names(&db, p, id_of(&db, a, "2")), ["B", "C"]);
    assert_eq!(names(&db, p, id_of(&db, c, "7")), ["A"]);
}

#[test]
fn ignores_hidden_unlinked_and_non_live() {
    let (mut db, p, a) = setup();
    let b = db.add_source(p, &source("B")).unwrap();
    let other = db.create_profile("O").unwrap();
    load(&mut db, a, vec![with_epg("1", "Arte", None)]);
    load(&mut db, b, vec![with_epg("2", "Arte HD", None)]);
    let id = id_of(&db, a, "1");
    assert_eq!(names(&db, p, id), ["B"]);
    assert!(names(&db, other, id).is_empty());
    db.set_stream_hidden(p, b, MediaKind::Live, "2", true)
        .unwrap();
    assert!(names(&db, p, id).is_empty());
    assert!(names(&db, p, 99_999).is_empty());
}
