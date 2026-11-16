use super::*;

fn q(height: i64) -> Quality {
    Quality {
        width: Some(height * 16 / 9),
        height: Some(height),
        fps: Some(50.0),
        video_codec: Some("h264".into()),
        bitrate: Some(4e6),
        startup_ms: Some(900),
        fails: 0,
    }
}

fn ids(db: &Db, p: ProfileId) -> Vec<i64> {
    let rows = db.live_variants(p, None).unwrap();
    rows.into_iter().map(|(r, _)| r.id).collect()
}

#[test]
fn save_and_failure_roundtrip() {
    let (mut db, p, s) = setup();
    db.replace_source_data(s, &playlist(&["c"], &[("k", "c", "TF1")]))
        .unwrap();
    let id = ids(&db, p)[0];
    assert!(db.quality_by_stream(&[id]).unwrap().is_empty());
    db.record_failure(s, "k").unwrap();
    db.record_failure(s, "k").unwrap();
    assert_eq!(db.quality_by_stream(&[id]).unwrap()[&id].fails, 2);
    db.save_quality(s, "k", &q(1080)).unwrap();
    assert_eq!(db.quality_by_stream(&[id]).unwrap()[&id], q(1080));
    db.record_failure(s, "k").unwrap();
    let got = &db.quality_by_stream(&[id]).unwrap()[&id];
    assert_eq!(got.fails, 1);
    assert_eq!(got.height, Some(1080));
}

#[test]
fn quality_survives_reimport() {
    let (mut db, p, s) = setup();
    let data = playlist(&["c"], &[("k", "c", "TF1")]);
    db.replace_source_data(s, &data).unwrap();
    db.save_quality(s, "k", &q(720)).unwrap();
    db.replace_source_data(s, &data).unwrap();
    let rows = db.live_variants(p, None).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].1, Some(q(720)));
    let id = rows[0].0.id;
    assert_eq!(db.quality_by_stream(&[id]).unwrap()[&id], q(720));
}

#[test]
fn live_variants_respects_visibility_and_profile() {
    let (mut db, p, a) = setup();
    db.replace_source_data(
        a,
        &playlist(
            &["c", "h"],
            &[("k1", "c", "One"), ("k2", "c", "Two"), ("k3", "h", "Three")],
        ),
    )
    .unwrap();
    db.set_stream_hidden(p, a, MediaKind::Live, "k2", true)
        .unwrap();
    db.set_category_hidden(p, a, MediaKind::Live, "h", true)
        .unwrap();
    let other = db.create_profile("Other").unwrap();
    let b = db.add_source(other, &source("B")).unwrap();
    db.replace_source_data(b, &playlist(&["c"], &[("x", "c", "X")]))
        .unwrap();
    let rows = db.live_variants(p, None).unwrap();
    let names: Vec<&str> = rows.iter().map(|(r, _)| r.name.as_str()).collect();
    assert_eq!(names, ["One"]);
    assert!(rows[0].1.is_none());
    assert!(db.live_variants(p, Some(b)).unwrap().is_empty());
}
