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

/// Per-file options for one play request. VOD gets only the catch-up "not live" flag.
pub fn file_options(settings: &PlayerSettings, media: MediaKind) -> Vec<(&'static str, String)> {
    let live = media == MediaKind::Live;
    let mut opts = Vec::new();
    if live && settings.live_low_latency {
        opts.extend(low_latency());
    }
    if script_path(settings).is_some() {
        let flag = if live { "yes" } else { "no" };
        opts.push(("script-opts", format!("live-catchup-live={flag}")));
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
    fn options_string_quotes_values() {
        let s = options_string(&[("a", "x,y".to_string()), ("b", "é".to_string())]);
        assert_eq!(s, "a=%3%x,y,b=%2%é");
    }
}
