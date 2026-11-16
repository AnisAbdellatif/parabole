mod account;
mod alternates;
mod copy_favorites;
mod epg;
mod groups;
mod import_scale;
mod new_categories;
mod overlay;
mod profiles;
mod progress;
mod quality;
mod scan;
mod search;
mod settings;
mod track_prefs;

use super::Db;
use crate::model::*;

pub fn source(name: &str) -> Source {
    Source {
        id: 0,
        name: name.into(),
        kind: SourceKind::M3uUrl,
        url: "http://example.test/list.m3u".into(),
        username: None,
        password: None,
        user_agent: None,
        mirrors: Vec::new(),
        output_format: None,
        last_refresh: None,
        last_error: None,
    }
}

pub fn cat(key: &str) -> ParsedCategory {
    ParsedCategory {
        media: MediaKind::Live,
        ext_key: key.into(),
        name: format!("Cat {key}"),
    }
}

pub fn stream(key: &str, category: &str, name: &str) -> ParsedStream {
    ParsedStream {
        media: MediaKind::Live,
        ext_key: key.into(),
        category_key: Some(category.into()),
        name: name.into(),
        ..Default::default()
    }
}

pub fn playlist(cats: &[&str], streams: &[(&str, &str, &str)]) -> ParsedPlaylist {
    ParsedPlaylist {
        categories: cats.iter().map(|c| cat(c)).collect(),
        streams: streams.iter().map(|(k, c, n)| stream(k, c, n)).collect(),
        ..Default::default()
    }
}

pub fn setup() -> (Db, ProfileId, SourceId) {
    let db = Db::open_in_memory().unwrap();
    let p = db.ensure_default_profile().unwrap();
    let s = db.add_source(p, &source("A")).unwrap();
    (db, p, s)
}

pub fn cat_keys(db: &Db, p: ProfileId, include_hidden: bool) -> Vec<String> {
    let rows = db
        .categories(p, None, MediaKind::Live, include_hidden)
        .unwrap();
    rows.into_iter().map(|c| c.key).collect()
}

#[test]
fn migrations_are_idempotent_on_reopen() {
    let dir = std::env::temp_dir().join(format!("iptv-db-test-{}", std::process::id()));
    let path = dir.join("nested").join("db.sqlite");
    let p = {
        let db = Db::open(&path).unwrap();
        let p = db.ensure_default_profile().unwrap();
        db.add_source(p, &source("A")).unwrap();
        p
    };
    let db = Db::open(&path).unwrap();
    assert_eq!(db.profiles().unwrap().len(), 1);
    assert_eq!(db.profile_sources(p).unwrap().len(), 1);
    let v: i64 = db
        .conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(v, 13);
    drop(db); // Windows can't delete a directory holding an open file
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn mirrors_roundtrip() {
    let (db, p, _) = setup();
    let mut s = source("M");
    s.mirrors = vec!["http://a:1".into(), "http://b:2".into()];
    let id = db.add_source(p, &s).unwrap();
    assert_eq!(db.source(id).unwrap().mirrors, s.mirrors);
    s.id = id;
    s.mirrors.clear();
    db.update_source(&s).unwrap();
    assert!(db.source(id).unwrap().mirrors.is_empty());
}
