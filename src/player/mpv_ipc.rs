use crate::model::PlayRequest;
use crate::player::argv::origin_header;
use crate::player::{PlayerSettings, live};
use anyhow::{Context, Result, bail};
use serde_json::json;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

const REPLY_TIMEOUT: Duration = Duration::from_millis(500);

pub fn socket_path() -> PathBuf {
    let pid = std::process::id();
    if cfg!(windows) {
        PathBuf::from(format!(r"\\.\pipe\parabole-mpv-{pid}"))
    } else {
        std::env::temp_dir().join(format!("parabole-mpv-{pid}.sock"))
    }
}

/// Stale socket files from a previous mpv would make mpv fail to bind.
pub fn remove_stale(path: &Path) {
    if !cfg!(windows) {
        let _ = std::fs::remove_file(path);
    }
}

/// `loadfile` with per-file options (mpv >= 0.38 argument order).
pub fn loadfile_payload(settings: &PlayerSettings, req: &PlayRequest) -> String {
    let mut opts = serde_json::Map::new();
    opts.insert("force-media-title".into(), json!(req.title));
    for (k, v) in live::file_options(settings, req.media) {
        opts.insert(k.into(), json!(v));
    }
    if let Some(s) = req.start_s {
        opts.insert("start".into(), json!(s.to_string()));
    }
    let h = &req.headers;
    if let Some(ua) = &h.user_agent {
        opts.insert("user-agent".into(), json!(ua));
    }
    if let Some(r) = &h.referrer {
        opts.insert("referrer".into(), json!(r));
    }
    if let Some(o) = &h.origin {
        opts.insert("http-header-fields".into(), json!(origin_header(o)));
    }
    json!({ "command": ["loadfile", req.url, "replace", -1, opts] }).to_string()
}

/// Asks the catch-up script to jump to the live edge (or reconnect).
pub fn go_live_payload() -> String {
    json!({ "command": ["script-message", "live-catchup-go-live"] }).to_string()
}

/// Reads mpv's reply to one command, skipping interleaved async events.
fn read_reply(reader: impl BufRead) -> Result<()> {
    let mut lines = reader.lines();
    for _ in 0..50 {
        let line = lines.next().context("mpv IPC closed")??;
        let Ok(v) = serde_json::from_str::<serde_json::Value>(&line) else {
            continue;
        };
        match v.get("error").and_then(serde_json::Value::as_str) {
            Some("success") => return Ok(()),
            Some(_) => bail!("mpv rejected loadfile"),
            None => continue,
        }
    }
    bail!("no reply from mpv")
}

#[cfg(unix)]
pub fn send(path: &Path, payload: &str) -> Result<()> {
    use std::os::unix::net::UnixStream;

    let mut stream = UnixStream::connect(path).context("mpv IPC connect failed")?;
    stream.set_write_timeout(Some(REPLY_TIMEOUT))?;
    stream.set_read_timeout(Some(REPLY_TIMEOUT))?;
    stream.write_all(payload.as_bytes())?;
    stream.write_all(b"\n")?;
    read_reply(BufReader::new(stream))
}

#[cfg(not(unix))]
pub fn send(path: &Path, payload: &str) -> Result<()> {
    let mut pipe = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .context("mpv IPC connect failed")?;
    pipe.write_all(payload.as_bytes())?;
    pipe.write_all(b"\n")?;
    let reader = pipe.try_clone()?;
    let (tx, rx) = std::sync::mpsc::channel();
    // named-pipe reads cannot time out; the thread ends when mpv closes the pipe
    std::thread::spawn(move || {
        let _ = tx.send(read_reply(BufReader::new(reader)));
    });
    rx.recv_timeout(REPLY_TIMEOUT)
        .unwrap_or_else(|_| bail!("no reply from mpv"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Headers, MediaKind};
    use serde_json::Value;

    fn req() -> PlayRequest {
        PlayRequest {
            url: "http://h/s.ts".into(),
            title: "T".into(),
            headers: Headers {
                user_agent: Some("UA".into()),
                referrer: None,
                origin: Some("http://o".into()),
            },
            media: MediaKind::Movie,
            ..Default::default()
        }
    }

    #[test]
    fn live_payload_carries_low_latency_options() {
        let mut r = req();
        r.media = MediaKind::Live;
        let v: Value =
            serde_json::from_str(&loadfile_payload(&PlayerSettings::default(), &r)).unwrap();
        let opts = &v["command"][4];
        assert_eq!(opts["cache-pause"], "no");
        assert_eq!(
            opts["demuxer-lavf-o-add"],
            "fflags=+nobuffer,live_start_index=-1"
        );
    }

    #[test]
    fn payload_format() {
        let v: Value =
            serde_json::from_str(&loadfile_payload(&PlayerSettings::default(), &req())).unwrap();
        assert_eq!(
            v,
            json!({"command": ["loadfile", "http://h/s.ts", "replace", -1, {
                "force-media-title": "T",
                "user-agent": "UA",
                "http-header-fields": "Origin: http://o"
            }]})
        );
    }

    fn reply(input: &str) -> Result<()> {
        read_reply(std::io::Cursor::new(input.to_owned()))
    }

    #[test]
    fn reply_skips_events_then_succeeds() {
        reply("{\"event\":\"x\"}\n{\"error\":\"success\",\"data\":null}\n").unwrap();
    }

    #[test]
    fn reply_rejection_is_error() {
        assert!(reply("{\"error\":\"invalid parameter\"}\n").is_err());
    }

    #[test]
    fn reply_skips_garbage() {
        reply("not json\n{\"error\":\"success\"}\n").unwrap();
    }

    #[test]
    fn reply_empty_input_is_error() {
        assert!(reply("").is_err());
    }

    #[test]
    fn go_live_payload_format() {
        let v: Value = serde_json::from_str(&go_live_payload()).unwrap();
        assert_eq!(
            v,
            json!({"command": ["script-message", "live-catchup-go-live"]})
        );
    }

    #[test]
    fn payload_is_single_line() {
        assert!(!loadfile_payload(&PlayerSettings::default(), &req()).contains('\n'));
    }

    #[cfg(unix)]
    fn serve_once(name: &str, reply: &'static str) -> (PathBuf, std::thread::JoinHandle<String>) {
        use std::io::{BufRead, BufReader, Write};
        let path =
            std::env::temp_dir().join(format!("iptv-ipc-{name}-{}.sock", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let listener = std::os::unix::net::UnixListener::bind(&path).unwrap();
        let h = std::thread::spawn(move || {
            let (mut s, _) = listener.accept().unwrap();
            let mut line = String::new();
            BufReader::new(s.try_clone().unwrap())
                .read_line(&mut line)
                .unwrap();
            s.write_all(reply.as_bytes()).unwrap();
            line
        });
        (path, h)
    }

    #[cfg(unix)]
    #[test]
    fn send_skips_events_and_accepts_success() {
        let (path, h) = serve_once(
            "ok",
            "{\"event\":\"x\"}\n{\"error\":\"success\",\"data\":null}\n",
        );
        send(&path, &loadfile_payload(&PlayerSettings::default(), &req())).unwrap();
        assert!(h.join().unwrap().contains("loadfile"));
        let _ = std::fs::remove_file(path);
    }

    #[cfg(unix)]
    #[test]
    fn send_reports_rejection_and_dead_socket() {
        let (path, h) = serve_once("err", "{\"error\":\"invalid parameter\"}\n");
        assert!(send(&path, "{}").is_err());
        h.join().unwrap();
        assert!(send(&path, "{}").is_err());
        let _ = std::fs::remove_file(path);
    }
}
