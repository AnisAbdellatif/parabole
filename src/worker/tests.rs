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

#[test]
fn second_refresh_of_same_source_is_skipped_until_done() {
    let db = Db::open_in_memory().unwrap();
    let profile = db.ensure_default_profile().unwrap();
    let fixture = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/sources/messy.m3u"
    );
    let id = db.add_source(profile, &m3u_source(fixture)).unwrap();
    let (cmd_tx, _cmd_rx) = channel();
    let (ev_tx, ev_rx) = channel();
    let mut w = Worker {
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
    };
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
