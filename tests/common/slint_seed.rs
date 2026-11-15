#![allow(dead_code)]
//! Scratch library and environment isolation shared by the Slint UI test binaries.
use iptv_player::db::Db;
use iptv_player::model::{Source, SourceKind};
use iptv_player::sources::m3u;
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
