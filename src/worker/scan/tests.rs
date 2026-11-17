use super::*;
use crate::model::{MediaKind, ParsedCategory, ParsedPlaylist, ParsedStream, Source, SourceKind};
use std::os::unix::fs::PermissionsExt;
use std::sync::mpsc::channel;

#[test]
fn step_backs_off_30s_when_a_db_write_fails() {
    let dir = std::env::temp_dir().join(format!("iptv-scan-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let bin = dir.join("ffprobe");
    std::fs::write(
        &bin,
        r#"#!/bin/sh
echo '{"streams":[{"codec_name":"h264","width":1280,"height":720,"avg_frame_rate":"25/1"}]}'
"#,
    )
    .unwrap();
    std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o755)).unwrap();

    let mut db = Db::open(&dir.join("db.sqlite")).unwrap();
    let profile = db.ensure_default_profile().unwrap();
    let source = db
        .add_source(
            profile,
            &Source {
                id: 0,
                name: "fixture".into(),
                kind: SourceKind::M3uUrl,
                url: "http://example.test/list.m3u".into(),
                username: None,
                password: None,
                user_agent: None,
                mirrors: Vec::new(),
                output_format: None,
                last_refresh: None,
                last_error: None,
            },
        )
        .unwrap();
    let data = ParsedPlaylist {
        categories: vec![ParsedCategory {
            media: MediaKind::Live,
            ext_key: "c".into(),
            name: "C".into(),
        }],
        streams: vec![ParsedStream {
            media: MediaKind::Live,
            ext_key: "k".into(),
            category_key: Some("c".into()),
            name: "One".into(),
            url: Some("http://example.test/live/1".into()),
            ..Default::default()
        }],
        ..Default::default()
    };
    db.replace_source_data(source, &data).unwrap();
    assert_eq!(db.scan_queue(profile, 20).unwrap().len(), 1);
    db.make_read_only();

    let ctl = Arc::new(ScanControl::default());
    *ctl.profile.lock().unwrap() = Some(profile);
    let (tx, _rx) = channel();
    let mut scanner = Scanner {
        db,
        ctl,
        tx,
        repaint: Arc::new(|| {}),
        last: None,
    };
    assert_eq!(scanner.step(&bin, profile), Duration::from_secs(30));
    let _ = std::fs::remove_dir_all(&dir);
}
