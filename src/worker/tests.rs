use super::*;
use crate::model::{MediaKind, SourceKind};
use std::time::Duration;

fn temp_db(name: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("iptv-worker-{name}-{}.db", std::process::id()));
    let _ = std::fs::remove_file(&p);
    p
}

fn wait_for<T>(h: &Handle, mut pick: impl FnMut(Event) -> Option<T>) -> T {
    loop {
        let ev = h.rx.recv_timeout(Duration::from_secs(20)).expect("event");
        if let Some(v) = pick(ev) {
            return v;
        }
    }
}

fn m3u_source(path: &str) -> Source {
    Source {
        id: 0,
        name: "fixture".into(),
        kind: SourceKind::M3uFile,
        url: path.into(),
        username: None,
        password: None,
        user_agent: None,
        mirrors: Vec::new(),
        output_format: None,
        last_refresh: None,
        last_error: None,
    }
}

#[test]
fn add_source_refreshes_and_imports() {
    let path = temp_db("add");
    let read = Db::open(&path).unwrap();
    let profile = read.ensure_default_profile().unwrap();
    let h = spawn(path.clone(), Arc::new(|| {})).unwrap();
    let fixture = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/sources/messy.m3u"
    );
    h.send(Command::AddSource(profile, m3u_source(fixture)));
    let res = wait_for(&h, |e| match e {
        Event::RefreshDone(_, r) => Some(r),
        _ => None,
    });
    assert_eq!(res, Ok(()));
    let cats = read
        .categories(profile, None, MediaKind::Live, true)
        .unwrap();
    assert!(!cats.is_empty());
    let rows = read
        .streams_in_category(
            profile,
            cats[0].source_id,
            MediaKind::Live,
            &cats[0].key,
            true,
        )
        .unwrap();
    assert!(!rows.is_empty());
    h.send(Command::Quit);
    let _ = std::fs::remove_file(path);
}

#[test]
fn bad_source_reports_error_and_play_without_player_fails_cleanly() {
    let path = temp_db("bad");
    let read = Db::open(&path).unwrap();
    let profile = read.ensure_default_profile().unwrap();
    let h = spawn(path.clone(), Arc::new(|| {})).unwrap();
    h.send(Command::AddSource(profile, m3u_source("/no/such/file.m3u")));
    let res = wait_for(&h, |e| match e {
        Event::RefreshDone(_, r) => Some(r),
        _ => None,
    });
    assert!(res.is_err());
    let src = &read.profile_sources(profile).unwrap()[0];
    assert!(src.last_error.is_some());
    h.send(Command::Play {
        profile,
        stream_id: 999,
        kind: None,
    });
    wait_for(&h, |e| matches!(e, Event::Error(_)).then_some(()));
    h.send(Command::Quit);
    let _ = std::fs::remove_file(path);
}

#[test]
fn play_failure_is_reported_not_fatal() {
    let path = temp_db("play");
    let read = Db::open(&path).unwrap();
    let profile = read.ensure_default_profile().unwrap();
    player_settings::save(
        &read,
        &crate::player::PlayerSettings {
            default: PlayerKind::Custom,
            ..Default::default()
        },
    )
    .unwrap();
    let h = spawn(path.clone(), Arc::new(|| {})).unwrap();
    let fixture = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/sources/messy.m3u"
    );
    h.send(Command::AddSource(profile, m3u_source(fixture)));
    wait_for(&h, |e| matches!(e, Event::RefreshDone(..)).then_some(()));
    let cats = read
        .categories(profile, None, MediaKind::Live, true)
        .unwrap();
    let rows = read
        .streams_in_category(
            profile,
            cats[0].source_id,
            MediaKind::Live,
            &cats[0].key,
            true,
        )
        .unwrap();
    h.send(Command::Play {
        profile,
        stream_id: rows[0].id,
        kind: None,
    });
    wait_for(&h, |e| matches!(e, Event::PlayFailed(_)).then_some(()));
    h.send(Command::Quit);
    let _ = std::fs::remove_file(path);
}

fn worker(db: Db) -> (Worker, Receiver<Event>) {
    let (cmd_tx, _cmd_rx) = channel();
    let (ev_tx, ev_rx) = channel();
    let w = Worker {
        scan: Arc::default(),
        db,
        db_path: PathBuf::new(),
        sports_busy: Arc::new(AtomicBool::new(false)),
        player: ExternalPlayer::default(),
        cmd_tx,
        ev_tx,
        repaint: Arc::new(|| {}),
        was_playing: false,
        auto_queue: VecDeque::new(),
        in_flight: HashSet::new(),
        epg_queue: VecDeque::new(),
        epg_busy: None,
    };
    (w, ev_rx)
}

#[test]
fn second_refresh_of_same_source_is_skipped_until_done() {
    let db = Db::open_in_memory().unwrap();
    let profile = db.ensure_default_profile().unwrap();
    let fixture = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/sources/messy.m3u"
    );
    let id = db.add_source(profile, &m3u_source(fixture)).unwrap();
    let (mut w, ev_rx) = worker(db);
    w.start_refresh(id).unwrap();
    w.start_refresh(id).unwrap();
    let started = |rx: &Receiver<Event>| {
        rx.try_iter()
            .filter(|e| matches!(e, Event::RefreshStarted(_)))
            .count()
    };
    assert_eq!(started(&ev_rx), 1);
    w.apply_fetched(id, Ok(ParsedPlaylist::default())).unwrap();
    w.start_refresh(id).unwrap();
    assert_eq!(started(&ev_rx), 1);
}

#[test]
fn epg_refresh_is_serialized_and_deduped() {
    let db = Db::open_in_memory().unwrap();
    let a = db.add_epg_guide("http://a.test/g.xml", "A").unwrap();
    let b = db.add_epg_guide("http://b.test/g.xml", "B").unwrap();
    let (mut w, _rx) = worker(db);
    w.epg_busy = Some(a);
    w.queue_epg(a);
    w.queue_epg(b);
    w.queue_epg(b);
    assert_eq!(w.epg_queue, [b]);
    w.apply_epg(a, Err("x".into()));
    // b's fetch thread is running: busy, queue drained
    assert_eq!(w.epg_busy, Some(b));
    assert!(w.epg_queue.is_empty());
}

#[test]
fn guide_without_urls_stamps_refreshed_and_reports_ok() {
    let db = Db::open_in_memory().unwrap();
    let profile = db.ensure_default_profile().unwrap();
    let sid = db.add_source(profile, &m3u_source("/x.m3u")).unwrap();
    let id = db.provider_guide(sid).unwrap().unwrap();
    let (mut w, rx) = worker(db);
    w.queue_epg(id);
    assert!(w.epg_busy.is_none());
    assert!(w.db.epg_guide(id).unwrap().refreshed_at.is_some());
    assert!(
        rx.try_iter()
            .any(|e| matches!(e, Event::EpgUpdated(i, Ok(())) if i == id))
    );
}

#[test]
fn disabled_guide_is_skipped() {
    let db = Db::open_in_memory().unwrap();
    let id = db.add_epg_guide("http://a.test/g.xml", "A").unwrap();
    db.set_epg_guide_enabled(id, false).unwrap();
    let (mut w, rx) = worker(db);
    w.queue_epg(id);
    assert!(w.epg_busy.is_none() && w.epg_queue.is_empty());
    assert!(rx.try_iter().next().is_none());
}

#[test]
fn relink_creates_links_and_keeps_manual() {
    use crate::model::*;
    let mut db = Db::open_in_memory().unwrap();
    let profile = db.ensure_default_profile().unwrap();
    let sid = db.add_source(profile, &m3u_source("/x.m3u")).unwrap();
    let stream = |k: &str, n: &str, e: &str| ParsedStream {
        media: MediaKind::Live,
        ext_key: k.into(),
        name: n.into(),
        epg_id: Some(e.into()),
        ..Default::default()
    };
    db.replace_source_data(
        sid,
        &ParsedPlaylist {
            streams: vec![stream("1", "A", "a.tv"), stream("2", "B", "zz")],
            ..Default::default()
        },
    )
    .unwrap();
    let g = db.add_epg_guide("http://a.test/g.xml", "G").unwrap();
    let chans = [GuideChannel {
        id: "a.tv".into(),
        names: vec!["A".into()],
    }];
    let progs = [Programme {
        channel: "a.tv".into(),
        start: 0,
        stop: i64::MAX / 2,
        title: "T".into(),
        descr: None,
    }];
    db.replace_guide(g, &chans, &progs, 1).unwrap();
    db.set_manual_link(sid, "2", g, "a.tv").unwrap();
    assert_eq!(super::epg::relink(&mut db).unwrap(), 1);
    assert_eq!(db.manual_link(sid, "2").unwrap(), Some((g, "a.tv".into())));
}
