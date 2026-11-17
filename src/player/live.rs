//! Per-file mpv options for live streams and the bundled catch-up script.
//! Options are per file (not `--profile=low-latency`) so they also work through IPC `loadfile`
//! and revert for VOD when an mpv instance is reused.

use super::PlayerSettings;
use crate::model::MediaKind;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

const SCRIPT_NAME: &str = "live-catchup.lua";
const SCRIPT: &str = include_str!("../../assets/mpv/live-catchup.lua");

/// Go live seeks inside the demuxer cache above this many buffered seconds; below it a seek gains nothing, so reconnect.
/// Keep equal to `seek_margin + 0.5` in `assets/mpv/live-catchup.lua`.
pub const GO_LIVE_MIN_S: f64 = 1.2;
/// Stay this far behind the end of the cache so the seek target is already buffered.
/// Keep equal to `seek_margin` in `assets/mpv/live-catchup.lua`.
pub const LIVE_EDGE_MARGIN_S: f64 = 0.7;

static SCRIPT_PATH: OnceLock<PathBuf> = OnceLock::new();

/// Writes the catch-up script into `dir` (the app data dir) and remembers its path.
pub fn install_script(dir: &Path) -> Result<()> {
    let path = dir.join(SCRIPT_NAME);
    std::fs::write(&path, SCRIPT).with_context(|| format!("write {}", path.display()))?;
    let _ = SCRIPT_PATH.set(path);
    Ok(())
}

/// Installed catch-up script, if enabled in settings and installed.
pub fn script_path(settings: &PlayerSettings) -> Option<&'static Path> {
    SCRIPT_PATH
        .get()
        .map(PathBuf::as_path)
        .filter(|_| settings.live_catchup)
}

/// What "go live" does for a live stream.
#[derive(Debug, PartialEq)]
pub enum GoLive {
    /// Seek to this absolute position inside the demuxer cache (instant, no reconnect).
    Seek(f64),
    /// Reconnect the stream.
    Reload,
}

/// Picks a cache seek when enough is buffered, else a reconnect; `edge_s` is `demuxer-cache-time`.
pub fn go_live_action(cache_s: Option<f64>, edge_s: Option<f64>) -> GoLive {
    match (cache_s, edge_s) {
        (Some(c), Some(e)) if c > GO_LIVE_MIN_S => GoLive::Seek(e - LIVE_EDGE_MARGIN_S),
        _ => GoLive::Reload,
    }
}

/// `vd-lavc-threads=1` from mpv's low-latency profile is left out: one thread cannot decode 4K HEVC in real time.
/// `demuxer-lavf-probe-info=nostreams` is left out too: mpv then fails to open HLS streams.
fn low_latency() -> Vec<(&'static str, String)> {
    let pairs = [
        ("audio-buffer", "0"),
        ("cache-pause", "no"),
        ("demuxer-lavf-o-add", "fflags=+nobuffer,live_start_index=-1"),
        ("demuxer-lavf-analyzeduration", "0.1"),
        ("video-sync", "audio"),
        ("interpolation", "no"),
        ("video-latency-hacks", "yes"),
        ("stream-buffer-size", "4k"),
    ];
    pairs.into_iter().map(|(k, v)| (k, v.to_string())).collect()
}

fn script_opts(live: bool, ok_s: f64) -> String {
    let flag = if live { "yes" } else { "no" };
    format!("live-catchup-live={flag},live-catchup-ok={ok_s}")
}

/// Per-file options for one play request. VOD gets only the catch-up "not live" flag.
pub fn file_options(settings: &PlayerSettings, media: MediaKind) -> Vec<(&'static str, String)> {
    let live = media == MediaKind::Live;
    let mut opts = Vec::new();
    if live && settings.live_low_latency {
        opts.extend(low_latency());
    }
    if script_path(settings).is_some() {
        opts.push(("script-opts", script_opts(live, settings.live_ok())));
    }
    opts
}

/// mpv list/option string for `loadfile ... key=value,...`; `%len%` prefixes make commas in values safe.
pub fn options_string(opts: &[(&str, String)]) -> String {
    opts.iter()
        .map(|(k, v)| format!("{k}=%{}%{v}", v.len()))
        .collect::<Vec<_>>()
        .join(",")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(o: &[(&str, String)]) -> Vec<String> {
        o.iter().map(|(k, _)| k.to_string()).collect()
    }

    #[test]
    fn live_gets_low_latency_options() {
        let o = file_options(&PlayerSettings::default(), MediaKind::Live);
        let k = keys(&o);
        assert!(k.contains(&"cache-pause".to_string()));
        assert!(!k.contains(&"vd-lavc-threads".to_string()));
        assert!(!k.contains(&"demuxer-lavf-probe-info".to_string()));
        let lavf = o.iter().find(|(k, _)| *k == "demuxer-lavf-o-add").unwrap();
        assert!(lavf.1.contains("live_start_index=-1"));
    }

    #[test]
    fn vod_and_disabled_get_no_latency_options() {
        let s = PlayerSettings::default();
        assert!(
            keys(&file_options(&s, MediaKind::Movie))
                .iter()
                .all(|k| k == "script-opts")
        );
        let off = PlayerSettings {
            live_low_latency: false,
            ..s
        };
        assert!(
            keys(&file_options(&off, MediaKind::Live))
                .iter()
                .all(|k| k == "script-opts")
        );
    }

    #[test]
    fn script_opts_carry_live_flag_and_ok_seconds() {
        let ok = PlayerSettings::default().live_ok();
        assert_eq!(
            script_opts(true, ok),
            "live-catchup-live=yes,live-catchup-ok=3"
        );
        assert!(script_opts(false, 2.5).contains("live-catchup-ok=2.5"));
    }

    #[test]
    fn go_live_seeks_only_with_enough_buffer() {
        assert_eq!(go_live_action(Some(5.0), Some(100.0)), GoLive::Seek(99.3));
        assert_eq!(go_live_action(Some(1.0), Some(100.0)), GoLive::Reload);
        assert_eq!(go_live_action(Some(5.0), None), GoLive::Reload);
        assert_eq!(go_live_action(None, None), GoLive::Reload);
    }

    #[test]
    fn options_string_quotes_values() {
        let s = options_string(&[("a", "x,y".to_string()), ("b", "é".to_string())]);
        assert_eq!(s, "a=%3%x,y,b=%2%é");
    }
}
