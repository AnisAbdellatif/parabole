use super::*;
use crate::model::{Headers, MediaKind};

#[test]
fn parses_stream_fields() {
    let q = parse(
        r#"{"programs":[],"streams":[{"codec_name":"h264","width":1920,"height":1080,
        "r_frame_rate":"50/1","avg_frame_rate":"25/1","bit_rate":"4500000"}],
        "format":{"bit_rate":"5000000"}}"#,
    )
    .unwrap();
    assert_eq!(q.height, Some(1080));
    assert_eq!(q.width, Some(1920));
    assert_eq!(q.fps, Some(25.0));
    assert_eq!(q.bitrate, Some(4.5e6));
    assert_eq!(q.video_codec.as_deref(), Some("h264"));
}

#[test]
fn falls_back_on_r_frame_rate_and_format_bitrate() {
    let q = parse(
        r#"{"streams":[{"codec_name":"hevc","width":1280,"height":720,
        "r_frame_rate":"60000/1001","avg_frame_rate":"0/0"}],"format":{"bit_rate":"3000000"}}"#,
    )
    .unwrap();
    assert!((q.fps.unwrap() - 59.94).abs() < 0.01);
    assert_eq!(q.bitrate, Some(3e6));
}

#[test]
fn no_video_stream_is_an_error() {
    assert!(parse(r#"{"streams":[],"format":{"bit_rate":"128000"}}"#).is_err());
    assert!(parse(r#"{"format":{}}"#).is_err());
    let hls = parse(r#"{"streams":[{"height":720}],"format":{"bit_rate":"9"}}"#).unwrap();
    assert_eq!(hls.bitrate, None);
    assert!(parse("not json").is_err());
}

#[test]
fn rejects_bad_url_without_echoing_it() {
    let req = PlayRequest {
        url: "file:///etc/secret-pass".into(),
        title: String::new(),
        headers: Headers::default(),
        media: MediaKind::Live,
        start_s: None,
    };
    match probe(Path::new("ffprobe"), &req, &|| false) {
        Err(ProbeError::Failed(e)) => assert!(!e.contains("secret-pass")),
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
#[ignore = "needs ffprobe and network"]
fn probes_real_stream() {
    let bin = find_ffprobe().expect("ffprobe");
    let req = PlayRequest {
        url: "https://test-streams.mux.dev/x36xhzz/x36xhzz.m3u8".into(),
        title: String::new(),
        headers: Headers::default(),
        media: MediaKind::Live,
        start_s: None,
    };
    let q = probe(&bin, &req, &|| false).unwrap();
    println!("{q:?}");
    assert!(q.height.unwrap_or(0) > 0);
}
