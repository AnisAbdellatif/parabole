use super::*;
use std::collections::HashSet;

fn prog(ch: &str, start: i64, stop: i64, title: &str) -> Programme {
    Programme {
        channel: ch.into(),
        start,
        stop,
        title: title.into(),
        descr: None,
    }
}

fn load(db: &mut Db, s: SourceId, streams: &[(&str, &str, Option<&str>, MediaKind)]) -> Vec<i64> {
    let mut data = playlist(&["c"], &[]);
    for (k, n, epg, media) in streams {
        let mut st = stream(k, "c", n);
        st.epg_id = epg.map(Into::into);
        st.media = *media;
        data.streams.push(st);
    }
    db.replace_source_data(s, &data).unwrap();
    streams
        .iter()
        .map(|(_, n, _, _)| {
            db.conn
                .query_row(
                    "SELECT id FROM streams WHERE name = ?1 AND source_id = ?2",
                    (n, s),
                    |r| r.get(0),
                )
                .unwrap()
        })
        .collect()
}

const L: MediaKind = MediaKind::Live;

#[test]
fn migration_creates_objects() {
    let db = Db::open_in_memory().unwrap();
    let n: i64 = db
        .conn
        .query_row("SELECT count(*) FROM epg_programmes", [], |r| r.get(0))
        .unwrap();
    assert_eq!(n, 0);
    db.conn
        .query_row("SELECT count(*) FROM stream_epg", [], |r| {
            r.get::<_, i64>(0)
        })
        .unwrap();
}

#[test]
fn replace_and_stale() {
    let (mut db, p, s) = setup();
    assert_eq!(db.epg_stale_sources(p, 100).unwrap(), vec![s]);
    db.replace_epg(s, &[prog("a", 0, 10, "old")], 500).unwrap();
    assert!(db.epg_stale_sources(p, 500).unwrap().is_empty());
    assert_eq!(db.epg_stale_sources(p, 501).unwrap(), vec![s]);
    db.replace_epg(
        s,
        &[prog("a", 20, 30, "new"), prog("a", 20, 31, "dup")],
        600,
    )
    .unwrap();
    let titles: Vec<String> = db
        .conn
        .prepare("SELECT title FROM epg_programmes")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(titles, vec!["new"]);
}

#[test]
fn channels_normalised() {
    let (mut db, _, s) = setup();
    load(
        &mut db,
        s,
        &[
            ("1", "A", Some(" TF1.fr "), L),
            ("2", "B", Some("  "), L),
            ("3", "M", Some("x"), MediaKind::Movie),
        ],
    );
    assert_eq!(
        db.epg_channels(s).unwrap(),
        HashSet::from(["tf1.fr".to_string()])
    );
}

#[test]
fn now_next_cases() {
    let (mut db, p, a) = setup();
    let b = db.add_source(p, &source("B")).unwrap();
    let ids = load(
        &mut db,
        a,
        &[
            ("1", "Tf1", Some(" TF1.fr "), L),
            ("2", "Gap", Some("gap"), L),
            ("3", "Mov", Some("tf1.fr"), MediaKind::Movie),
        ],
    );
    let idb = load(&mut db, b, &[("1", "Other", Some("tf1.fr"), L)]);
    db.replace_epg(
        a,
        &[
            prog("tf1.fr", 0, 100, "now"),
            prog("tf1.fr", 100, 200, "next"),
            prog("tf1.fr", 200, 300, "later"),
            prog("gap", 0, 50, "past"),
            prog("gap", 150, 200, "future"),
        ],
        1,
    )
    .unwrap();
    db.replace_epg(b, &[prog("tf1.fr", 50, 400, "bnow")], 1)
        .unwrap();
    let all = [ids[0], ids[1], ids[2], idb[0]];
    let m = db.epg_now_next(&all, 60).unwrap();
    assert_eq!(m.len(), 2);
    assert_eq!(m[&ids[0]].now.title, "now");
    assert_eq!(m[&ids[0]].next.as_ref().unwrap().title, "next");
    assert_eq!(m[&idb[0]].now.title, "bnow");
    assert!(m[&idb[0]].next.is_none());
    assert!(db.epg_now_next(&[], 60).unwrap().is_empty());
    assert_eq!(
        db.epg_now_next(&all, 100).unwrap()[&ids[0]].now.title,
        "next"
    );
}

#[test]
fn schedule_and_prune() {
    let (mut db, _, s) = setup();
    let ids = load(&mut db, s, &[("1", "A", Some("c"), L)]);
    db.replace_epg(
        s,
        &[
            prog("c", 0, 10, "a"),
            prog("c", 10, 20, "b"),
            prog("c", 20, 30, "c"),
        ],
        1,
    )
    .unwrap();
    let t = |f, to| -> Vec<String> {
        db.epg_schedule(ids[0], f, to)
            .unwrap()
            .into_iter()
            .map(|p| p.title)
            .collect()
    };
    assert_eq!(t(10, 20), vec!["b"]);
    assert_eq!(t(9, 21), vec!["a", "b", "c"]);
    assert!(t(30, 40).is_empty());
    db.epg_prune(10).unwrap();
    assert_eq!(t(0, 100), vec!["a", "b", "c"]);
    db.epg_prune(11).unwrap();
    assert_eq!(t(0, 100), vec!["b", "c"]);
}

#[test]
fn source_delete_cascades() {
    let (mut db, _, s) = setup();
    db.replace_epg(s, &[prog("c", 0, 10, "a")], 1).unwrap();
    db.delete_source(s).unwrap();
    let n: i64 = db
        .conn
        .query_row("SELECT count(*) FROM epg_programmes", [], |r| r.get(0))
        .unwrap();
    assert_eq!(n, 0);
}
