use super::*;
use rusqlite::Connection;

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

fn gch(id: &str, names: &[&str]) -> GuideChannel {
    GuideChannel {
        id: id.into(),
        names: names.iter().map(|n| n.to_string()).collect(),
    }
}

fn link(s: SourceId, key: &str, g: GuideId, ch: &str) -> EpgLink {
    EpgLink {
        source_id: s,
        stream_key: key.into(),
        guide_id: g,
        channel: ch.into(),
    }
}

fn count(db: &Db, table: &str) -> i64 {
    db.conn
        .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
        .unwrap()
}

#[test]
fn migration_fresh_and_from_v9() {
    let db = Db::open_in_memory().unwrap();
    assert_eq!(count(&db, "epg_guides"), 0);
    assert_eq!(count(&db, "stream_epg"), 0);

    let conn = Connection::open_in_memory().unwrap();
    for sql in &crate::db::migrate::MIGRATIONS[..9] {
        conn.execute_batch(sql).unwrap();
    }
    conn.pragma_update(None, "user_version", 9).unwrap();
    conn.execute(
        "INSERT INTO sources(name, kind, url, epg_refreshed_at) VALUES ('Old', ?1, 'http://x', 5)",
        [SourceKind::M3uUrl.as_str()],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO epg_programmes VALUES (1, 'a', 0, 10, 't', NULL)",
        [],
    )
    .unwrap();
    let db = Db::init(conn).unwrap();
    let guides = db.epg_guides().unwrap();
    assert_eq!(guides.len(), 1);
    assert_eq!(guides[0].name, "Old");
    assert_eq!(guides[0].source_id, Some(1));
    assert_eq!(count(&db, "epg_programmes"), 0);
}

#[test]
fn add_source_creates_provider_guide() {
    let (db, _, s) = setup();
    let g = db.provider_guide(s).unwrap().unwrap();
    let guide = db.epg_guide(g).unwrap();
    assert_eq!(
        (guide.name.as_str(), guide.source_id, guide.enabled),
        ("A", Some(s), true)
    );
    assert_eq!(db.provider_guide(999).unwrap(), None);
}

#[test]
fn guide_management() {
    let (db, _, s) = setup();
    let prov = db.provider_guide(s).unwrap().unwrap();
    assert!(db.add_epg_guide("ftp://x/guide.xml", "bad").is_err());
    assert!(db.add_epg_guide("not a url", "bad").is_err());
    let a = db.add_epg_guide("http://x/a.xml", "a").unwrap();
    let b = db.add_epg_guide("https://x/b.xml", "b").unwrap();
    let ids = |db: &Db| -> Vec<GuideId> { db.epg_guides().unwrap().iter().map(|g| g.id).collect() };
    assert_eq!(ids(&db), [prov, a, b]);
    db.move_epg_guide(b, -1).unwrap();
    assert_eq!(ids(&db), [prov, b, a]);
    db.move_epg_guide(prov, 1).unwrap();
    assert_eq!(ids(&db), [b, prov, a]);
    db.move_epg_guide(a, 5).unwrap();
    assert_eq!(ids(&db), [b, prov, a]);
    db.set_epg_guide_enabled(a, false).unwrap();
    assert!(!db.epg_guide(a).unwrap().enabled);
    assert!(db.delete_epg_guide(prov).is_err());
    db.delete_epg_guide(a).unwrap();
    assert_eq!(ids(&db), [b, prov]);
}

#[test]
fn replace_guide_counts_and_replaces() {
    let (mut db, _, s) = setup();
    let g = db.provider_guide(s).unwrap().unwrap();
    let chans = [gch("a", &["A One", "Alpha"]), gch("b", &["B"])];
    let progs = [
        prog("a", 0, 10, "x"),
        prog("a", 10, 20, "y"),
        prog("a", 10, 21, "dup"),
    ];
    db.replace_guide(g, &chans, &progs, 100).unwrap();
    let n: Vec<(String, i64)> = db
        .conn
        .prepare("SELECT channel, programmes FROM epg_guide_channels ORDER BY channel")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(n, [("a".into(), 2), ("b".into(), 0)]);
    assert_eq!(db.epg_guide(g).unwrap().refreshed_at, Some(100));
    db.replace_guide(g, &[gch("c", &["C"])], &[prog("c", 0, 5, "z")], 200)
        .unwrap();
    assert_eq!(count(&db, "epg_guide_channels"), 1);
    assert_eq!(count(&db, "epg_programmes"), 1);
    db.epg_prune(6).unwrap();
    assert_eq!(count(&db, "epg_programmes"), 0);
}

#[test]
fn stale_and_error_stamp() {
    let (mut db, _, s) = setup();
    let g = db.provider_guide(s).unwrap().unwrap();
    let u = db.add_epg_guide("http://x/a.xml", "u").unwrap();
    assert_eq!(db.stale_epg_guides(100).unwrap(), [g, u]);
    db.replace_guide(g, &[], &[], 500).unwrap();
    assert_eq!(db.stale_epg_guides(500).unwrap(), [u]);
    assert_eq!(db.stale_epg_guides(501).unwrap(), [g, u]);
    db.set_epg_guide_error(u, "boom", 600).unwrap();
    let guide = db.epg_guide(u).unwrap();
    assert_eq!(
        (guide.last_error.as_deref(), guide.refreshed_at),
        (Some("boom"), Some(600))
    );
    assert_eq!(db.stale_epg_guides(600).unwrap(), [g]);
    db.set_epg_guide_enabled(g, false).unwrap();
    assert_eq!(db.stale_epg_guides(10_000).unwrap(), [u]);
    db.replace_guide(u, &[], &[], 700).unwrap();
    assert_eq!(db.epg_guide(u).unwrap().last_error, None);
}

#[test]
fn stream_keys_and_candidates() {
    let (mut db, _, s) = setup();
    let g = db.provider_guide(s).unwrap().unwrap();
    load(
        &mut db,
        s,
        &[
            ("k1", "One", Some("A"), L),
            ("k2", "Film", None, MediaKind::Movie),
        ],
    );
    let keys = db.epg_stream_keys().unwrap();
    assert_eq!(keys.len(), 1);
    assert_eq!(
        (keys[0].stream_key.as_str(), keys[0].epg_id.as_deref()),
        ("k1", Some("A"))
    );
    db.replace_guide(
        g,
        &[gch("a", &["A One", "Alpha"]), gch("empty", &["E"])],
        &[prog("a", 0, 10, "x")],
        1,
    )
    .unwrap();
    assert_eq!(
        db.epg_candidates().unwrap(),
        [(
            g,
            "a".to_string(),
            vec!["A One".to_string(), "Alpha".to_string()]
        )]
    );
    db.set_epg_guide_enabled(g, false).unwrap();
    assert!(db.epg_candidates().unwrap().is_empty());
}

#[test]
fn auto_links_keep_manual() {
    let (mut db, _, s) = setup();
    let g = db.provider_guide(s).unwrap().unwrap();
    load(
        &mut db,
        s,
        &[("k1", "One", None, L), ("k2", "Two", None, L)],
    );
    db.set_manual_link(s, "k1", g, "m").unwrap();
    db.replace_auto_links(&[link(s, "k1", g, "auto"), link(s, "k2", g, "auto2")])
        .unwrap();
    assert_eq!(db.manual_link(s, "k1").unwrap(), Some((g, "m".into())));
    assert_eq!(db.manual_link(s, "k2").unwrap(), None);
    assert_eq!(count(&db, "epg_links"), 2);
    db.replace_auto_links(&[]).unwrap();
    assert_eq!(count(&db, "epg_links"), 1);
    db.clear_manual_link(s, "k1").unwrap();
    assert_eq!(count(&db, "epg_links"), 0);
}

#[test]
fn now_next_through_links() {
    let (mut db, p, s) = setup();
    let g1 = db.provider_guide(s).unwrap().unwrap();
    let g2 = db.add_epg_guide("http://x/b.xml", "b").unwrap();
    let ids = load(
        &mut db,
        s,
        &[
            ("k1", "One", None, L),
            ("k2", "Two", None, L),
            ("k3", "None", None, L),
        ],
    );
    let _ = p;
    db.replace_guide(
        g1,
        &[gch("tf1", &["TF1"])],
        &[
            prog("tf1", 0, 100, "g1 now"),
            prog("tf1", 100, 200, "g1 next"),
        ],
        1,
    )
    .unwrap();
    db.replace_guide(
        g2,
        &[gch("tf1", &["TF1"])],
        &[prog("tf1", 0, 100, "g2 now")],
        1,
    )
    .unwrap();
    db.replace_auto_links(&[link(s, "k1", g1, "tf1"), link(s, "k2", g2, "tf1")])
        .unwrap();
    let nn = db.epg_now_next(&ids, 50).unwrap();
    assert_eq!(nn[&ids[0]].now.title, "g1 now");
    assert_eq!(nn[&ids[0]].next.as_ref().unwrap().title, "g1 next");
    assert_eq!(nn[&ids[1]].now.title, "g2 now");
    assert!(!nn.contains_key(&ids[2]));
    assert_eq!(db.epg_schedule(ids[0], 0, 1000).unwrap().len(), 2);
    assert_eq!(db.epg_schedule(ids[1], 0, 1000).unwrap().len(), 1);
    db.set_epg_guide_enabled(g2, false).unwrap();
    assert!(!db.epg_now_next(&ids, 50).unwrap().contains_key(&ids[1]));
    assert!(db.epg_schedule(ids[1], 0, 1000).unwrap().is_empty());
}

#[test]
fn links_survive_refresh_and_source_delete() {
    let (mut db, p, s) = setup();
    let g = db.provider_guide(s).unwrap().unwrap();
    let ids = load(&mut db, s, &[("k1", "One", None, L)]);
    db.replace_guide(g, &[gch("a", &["A"])], &[prog("a", 0, 100, "x")], 1)
        .unwrap();
    db.replace_auto_links(&[link(s, "k1", g, "a")]).unwrap();
    let new_ids = load(&mut db, s, &[("k1", "One", None, L)]);
    assert_eq!(db.epg_now_next(&new_ids, 50).unwrap().len(), 1);
    let _ = ids;
    assert_eq!(db.epg_coverage().unwrap()[&g], 1);
    let g2 = db.add_epg_guide("http://x/b.xml", "b").unwrap();
    db.set_manual_link(s, "k1", g2, "z").unwrap();
    db.delete_epg_guide(g2).unwrap();
    assert_eq!(count(&db, "epg_links"), 0);
    db.replace_auto_links(&[link(s, "k1", g, "a")]).unwrap();
    db.delete_source(s).unwrap();
    assert_eq!(count(&db, "epg_links"), 0);
    assert_eq!(count(&db, "epg_guides"), 0);
    let _ = p;
}

#[test]
fn search_and_coverage() {
    let (mut db, _, s) = setup();
    let g = db.provider_guide(s).unwrap().unwrap();
    let u = db.add_epg_guide("http://x/b.xml", "b").unwrap();
    db.replace_guide(
        g,
        &[gch("tf1.fr", &["TF1 HD", "Tele"]), gch("arte", &["Arte"])],
        &[],
        1,
    )
    .unwrap();
    db.replace_guide(u, &[gch("x", &["Other tf1"])], &[], 1)
        .unwrap();
    let r = db.search_guide_channels("TF1", 10).unwrap();
    assert_eq!(
        r,
        [
            (
                g,
                "A".to_string(),
                "tf1.fr".to_string(),
                "TF1 HD".to_string()
            ),
            (u, "b".to_string(), "x".to_string(), "Other tf1".to_string())
        ]
    );
    assert_eq!(db.search_guide_channels("tf1", 1).unwrap().len(), 1);
    assert_eq!(db.search_guide_channels("arte", 10).unwrap().len(), 1);
    assert!(db.search_guide_channels("%", 10).unwrap().is_empty());
    db.set_epg_guide_enabled(u, false).unwrap();
    assert_eq!(db.search_guide_channels("tf1", 10).unwrap().len(), 1);
    assert!(db.epg_coverage().unwrap().is_empty());
}

#[test]
fn duplicate_add_returns_same_id_and_delete_added_keeps_provider() {
    let (db, _, sid) = setup();
    let a = db.add_epg_guide("https://a.test/g.xml", "A").unwrap();
    assert_eq!(db.add_epg_guide("https://a.test/g.xml", "A2").unwrap(), a);
    db.add_epg_guide("https://b.test/g.xml", "B").unwrap();
    assert_eq!(db.delete_added_epg_guides().unwrap(), 2);
    let left = db.epg_guides().unwrap();
    assert_eq!(left.len(), 1);
    assert_eq!(left[0].source_id, Some(sid));
}
