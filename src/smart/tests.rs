use super::claimed::{claimed, display_name};
use super::*;
use crate::model::MediaKind;

fn row(id: i64, name: &str) -> StreamRow {
    StreamRow {
        id,
        source_id: 1,
        media: MediaKind::Live,
        key: id.to_string(),
        name: name.into(),
        num: None,
        logo: None,
        favorite: false,
        hidden: false,
        missing: false,
        progress: None,
        resume_episode: None,
    }
}

fn v(id: i64, name: &str, quality: Option<Quality>) -> Variant {
    Variant {
        row: row(id, name),
        quality,
    }
}

fn measured(height: i64, fails: i64) -> Option<Quality> {
    Some(Quality {
        height: Some(height),
        fails,
        ..Default::default()
    })
}

fn ids(ch: &Channel) -> Vec<i64> {
    ch.variants.iter().map(|v| v.row.id).collect()
}

#[test]
fn groups_same_channel_across_sources() {
    let chans = group(vec![
        v(1, "FR| TF1 FHD", None),
        v(2, "TF1 HD", None),
        v(3, "TF2", None),
        v(4, "TF1 ᴴᴰ", None),
    ]);
    assert_eq!(chans.len(), 2);
    assert_eq!(chans[0].name, "TF1");
    assert_eq!(ids(&chans[0]), [1, 2, 4]);
    assert_eq!(chans[1].name, "TF2");
    assert_eq!(badge(&chans[0]), "≈1080p (from name) · 3 versions");
}

#[test]
fn empty_key_is_its_own_channel() {
    let chans = group(vec![v(1, "HD", None), v(2, "HD", None)]);
    assert_eq!(chans.len(), 2);
    assert_eq!(chans[0].name, "HD");
}

#[test]
fn ranking() {
    let chans = group(vec![
        v(1, "TF1", measured(1080, 0)),
        v(2, "TF1 4K", None),
        v(3, "TF1", measured(2160, 2)),
        v(4, "TF1 FHD", None),
        v(5, "TF1", None),
    ]);
    // Claimed 2160 beats measured 1080; measured beats claimed at equal height; failed last.
    assert_eq!(ids(&chans[0]), [2, 1, 4, 5, 3]);
}

#[test]
fn ranking_fps_then_startup() {
    let q = |fps: f64, ms: Option<i64>| {
        Some(Quality {
            height: Some(720),
            fps: Some(fps),
            startup_ms: ms,
            ..Default::default()
        })
    };
    let chans = group(vec![
        v(1, "A", q(25.0, Some(500))),
        v(2, "A", q(50.0, None)),
        v(3, "A", q(50.0, Some(800))),
    ]);
    assert_eq!(ids(&chans[0]), [3, 2, 1]);
}

#[test]
fn claimed_tags() {
    assert_eq!(claimed("TF1 FHD"), (Some(1080), None));
    assert_eq!(claimed("ESPN 4K 60FPS"), (Some(2160), Some(60.0)));
    assert_eq!(claimed("BBC HD/SD"), (Some(720), None));
    assert_eq!(claimed("TF1ᶠᴴᴰ"), (Some(1080), None));
    assert_eq!(claimed("Arte ᵁᴴᴰ"), (Some(2160), None));
    assert_eq!(claimed("Canal 8K"), (Some(4320), None));
    assert_eq!(claimed("Hadopi"), (None, None));
}

#[test]
fn display_names() {
    assert_eq!(display_name("FR| TF1 FHD"), "TF1");
    assert_eq!(display_name("TF1 ᴴᴰ"), "TF1");
    assert_eq!(display_name("TF1ᴴᴰ"), "TF1");
    assert_eq!(display_name("◉ Eurosport 1 (1080p) | 50FPS"), "Eurosport 1");
    assert_eq!(display_name("UK: BBC One - [HD]"), "BBC One");
    assert_eq!(display_name(" HD "), "HD");
}

#[test]
fn quality_labels() {
    let mut q = Quality {
        height: Some(1080),
        fps: Some(49.97),
        video_codec: Some("hevc".into()),
        bitrate: Some(6_240_000.0),
        ..Default::default()
    };
    assert_eq!(quality_label(&q).unwrap(), "1080p50 · HEVC · 6.2 Mb/s");
    q.fps = Some(0.0);
    q.video_codec = Some("vp9".into());
    q.bitrate = None;
    q.fails = 1;
    assert_eq!(quality_label(&q).unwrap(), "1080p · VP9 · failed last time");
    q.video_codec = None;
    q.fps = None;
    q.fails = 0;
    assert_eq!(quality_label(&q).unwrap(), "1080p");
    assert_eq!(quality_label(&Quality::default()), None);
}

#[test]
fn buckets() {
    let chans = group(vec![
        v(1, "A", measured(2160, 0)),
        v(2, "B 1080p 50fps", None),
        v(3, "C", measured(576, 0)),
        v(4, "D", None),
    ]);
    let names = |b: Bucket| -> Vec<&str> {
        let hits = chans.iter().filter(|c| b.contains(c));
        hits.map(|c| c.name.as_str()).collect()
    };
    assert_eq!(names(Bucket::All).len(), 4);
    assert_eq!(names(Bucket::Uhd), ["A"]);
    assert_eq!(names(Bucket::FullHd), ["B"]);
    assert!(names(Bucket::Hd).is_empty());
    assert_eq!(names(Bucket::Sd), ["C"]);
    assert_eq!(names(Bucket::HighFps), ["B"]);
    assert_eq!(names(Bucket::Unknown), ["D"]);
    for b in Bucket::ALL {
        assert_eq!(Bucket::from_index(b.index()), Some(b));
    }
    assert_eq!(Bucket::from_index(7), None);
}
