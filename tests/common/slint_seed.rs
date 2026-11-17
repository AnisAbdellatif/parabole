#![allow(dead_code)]
//! Scratch library and environment isolation shared by the Slint UI test binaries.
use parabole::db::Db;
use parabole::model::{Quality, Source, SourceKind};
use parabole::sources::m3u;
use std::path::{Path, PathBuf};

const FIXTURE: &[u8] = include_bytes!("../fixtures/slint-ui.m3u");

/// Fresh scratch dir for one test binary; removed with `cleanup`.
pub fn scratch_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("iptv-slint-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

pub fn cleanup(dir: &Path) {
    let _ = std::fs::remove_dir_all(dir);
}

/// Points every user dir (config, data, image cache) at `dir`; call first in the test, before any thread starts.
pub fn isolate_home(dir: &Path) {
    // SAFETY: called at the start of a single-test binary, before any other thread exists.
    unsafe {
        std::env::set_var("HOME", dir);
        std::env::set_var("XDG_DATA_HOME", dir.join("data"));
        std::env::set_var("XDG_CACHE_HOME", dir.join("cache"));
        std::env::set_var("XDG_CONFIG_HOME", dir.join("config"));
    }
}

/// Creates `dir/library.db`: one profile, one M3U source (unreachable URL) holding the fixture, auto-refresh off.
pub fn seed(dir: &Path) -> PathBuf {
    let path = dir.join("library.db");
    let mut db = Db::open(&path).expect("open scratch db");
    let profile = db.ensure_default_profile().expect("default profile");
    db.set_setting(None, "auto_refresh_hours", "0")
        .expect("disable auto refresh");
    let source = db
        .add_source(
            profile,
            &Source {
                id: 0,
                name: "Fixture".into(),
                kind: SourceKind::M3uUrl,
                url: "http://127.0.0.1:9/none.m3u".into(),
                username: None,
                password: None,
                user_agent: None,
                mirrors: Vec::new(),
                output_format: None,
                last_refresh: None,
                last_error: None,
            },
        )
        .expect("add source");
    let playlist = m3u::parse(FIXTURE).expect("parse fixture");
    db.replace_source_data(source, &playlist)
        .expect("import fixture");
    path
}

pub fn add_profile(db_path: &Path, name: &str) {
    let db = Db::open(db_path).expect("open scratch db");
    db.create_profile(name).expect("create profile");
}

/// Writes the Slint UI state file next to the DB (JSON of `Persisted`).
pub fn write_state(db_path: &Path, json: &str) {
    std::fs::write(db_path.with_file_name("slint-ui.json"), json).expect("write state");
}

const BACKUP: &[u8] = b"#EXTM3U
#EXTINF:-1 tvg-id=\"b1\" group-title=\"News\",World News
http://backup.test/live/u/p/1.ts
#EXTINF:-1 tvg-id=\"b2\" group-title=\"News\",Channel One
http://backup.test/live/u/p/2.ts
#EXTINF:-1 tvg-id=\"b3\" group-title=\"News\",Daily Brief
http://backup.test/live/u/p/3.ts
";

/// Adds measured qualities for the fixture's News channels plus a second source "Backup"
/// carrying three of the same channels, so the Smart tab has buckets and grouped versions.
pub fn seed_smart(db_path: &Path) {
    let mut db = Db::open(db_path).expect("open scratch db");
    let profile = db.ensure_default_profile().expect("default profile");
    let fixture = db.profile_sources(profile).expect("sources")[0].id;
    let backup = db
        .add_source(
            profile,
            &Source {
                id: 0,
                name: "Backup".into(),
                kind: SourceKind::M3uUrl,
                url: "http://127.0.0.1:9/backup.m3u".into(),
                username: None,
                password: None,
                user_agent: None,
                mirrors: Vec::new(),
                output_format: None,
                last_refresh: None,
                last_error: None,
            },
        )
        .expect("add backup");
    let playlist = m3u::parse(BACKUP).expect("parse backup");
    db.replace_source_data(backup, &playlist)
        .expect("import backup");
    let q = |height, fps, codec: &str, mbps: f64, ms| Quality {
        width: Some(height * 16 / 9),
        height: Some(height),
        fps: Some(fps),
        video_codec: Some(codec.into()),
        bitrate: Some(mbps * 1e6),
        startup_ms: Some(ms),
        fails: 0,
    };
    let rows = [
        (fixture, "ch1", q(2160, 50.0, "hevc", 18.0, 900)),
        (fixture, "ch2", q(1080, 50.0, "h264", 8.2, 700)),
        (fixture, "ch3", q(1080, 25.0, "h264", 6.1, 800)),
        (fixture, "ch4", q(720, 25.0, "h264", 3.4, 600)),
        (fixture, "ch5", q(576, 25.0, "mpeg2video", 2.0, 500)),
        (backup, "b1", q(1080, 25.0, "h264", 5.0, 650)),
        (backup, "b2", q(720, 25.0, "h264", 3.0, 550)),
        (backup, "b3", q(1080, 50.0, "hevc", 7.5, 750)),
    ];
    for (source, key, quality) in &rows {
        db.save_quality(*source, key, quality)
            .expect("save quality");
    }
}
