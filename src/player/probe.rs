//! Measures a live stream's video quality with ffprobe, without playing it.

use super::locate::{env_dir, locate};
use super::validate::clean_request;
use crate::model::{PlayRequest, Quality};
use anyhow::{Context, Result, anyhow};
use serde::Deserialize;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Debug)]
pub enum ProbeError {
    Cancelled,
    Failed(String),
}

/// PATH first, then the usual install dirs (GUI apps on macOS don't get Homebrew in PATH).
pub fn find_ffprobe() -> Option<PathBuf> {
    let defaults = if cfg!(target_os = "macos") {
        ["/opt/homebrew/bin/ffprobe", "/usr/local/bin/ffprobe"]
            .map(PathBuf::from)
            .to_vec()
    } else if cfg!(windows) {
        [
            env_dir("ProgramFiles", r"ffmpeg\bin\ffprobe.exe"),
            env_dir("USERPROFILE", r"scoop\shims\ffprobe.exe"),
        ]
        .into_iter()
        .flatten()
        .collect()
    } else {
        ["/usr/bin/ffprobe", "/snap/bin/ffprobe"]
            .map(PathBuf::from)
            .to_vec()
    };
    locate(None, "ffprobe", defaults)
}

fn failed(e: impl std::fmt::Display) -> ProbeError {
    ProbeError::Failed(e.to_string())
}

/// Probes `req.url`, polling `cancel` every 100 ms. Error text never contains the URL.
pub fn probe(
    bin: &Path,
    req: &PlayRequest,
    cancel: &dyn Fn() -> bool,
) -> Result<Quality, ProbeError> {
    let req = clean_request(req).map_err(failed)?;
    let mut cmd = Command::new(bin);
    cmd.args(["-v", "error", "-hide_banner", "-rw_timeout", "8000000"])
        .args(["-analyzeduration", "3000000", "-probesize", "3000000"]);
    if let Some(ua) = &req.headers.user_agent {
        cmd.arg("-user_agent").arg(ua);
    }
    let mut headers = String::new();
    if let Some(r) = &req.headers.referrer {
        headers.push_str(&format!("Referer: {r}\r\n"));
    }
    if let Some(o) = &req.headers.origin {
        headers.push_str(&format!("Origin: {o}\r\n"));
    }
    if !headers.is_empty() {
        cmd.arg("-headers").arg(headers);
    }
    cmd.args(["-select_streams", "v:0", "-show_entries"])
        .arg("stream=codec_name,width,height,avg_frame_rate,r_frame_rate,bit_rate:format=bit_rate")
        .args(["-of", "json"])
        .arg(&req.url)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    let start = Instant::now();
    let mut child = cmd
        .spawn()
        .map_err(|e| failed(format!("start ffprobe: {e}")))?;
    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| failed("no ffprobe stdout"))?;
    // Read on a thread so a full pipe buffer can't block ffprobe from exiting.
    let reader = std::thread::spawn(move || {
        let mut out = String::new();
        stdout.read_to_string(&mut out).map(|_| out)
    });
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {}
            Err(e) => return Err(failed(format!("wait for ffprobe: {e}"))),
        }
        let timed_out = start.elapsed() > TIMEOUT;
        if timed_out || cancel() {
            let _ = child.kill();
            let _ = child.wait();
            let _ = reader.join();
            return Err(if timed_out {
                failed("ffprobe timed out")
            } else {
                ProbeError::Cancelled
            });
        }
        std::thread::sleep(Duration::from_millis(100));
    };
    let elapsed = start.elapsed().as_millis() as i64;
    let out = reader
        .join()
        .map_err(|_| failed("ffprobe reader panicked"))?
        .map_err(|e| failed(format!("read ffprobe output: {e}")))?;
    if !status.success() {
        return Err(failed(format!("ffprobe exited with {status}")));
    }
    let mut q = parse(&out).map_err(|e| failed(format!("{e:#}")))?;
    q.startup_ms = Some(elapsed);
    Ok(q)
}

#[derive(Deserialize)]
struct Output {
    #[serde(default)]
    streams: Vec<Stream>,
    format: Option<Format>,
}

#[derive(Deserialize)]
struct Stream {
    codec_name: Option<String>,
    width: Option<i64>,
    height: Option<i64>,
    avg_frame_rate: Option<String>,
    r_frame_rate: Option<String>,
    bit_rate: Option<String>,
}

#[derive(Deserialize)]
struct Format {
    bit_rate: Option<String>,
}

fn rate(s: Option<&str>) -> Option<f64> {
    let (n, d) = s?.split_once('/')?;
    let (n, d): (f64, f64) = (n.parse().ok()?, d.parse().ok()?);
    (n > 0.0 && d > 0.0).then(|| n / d)
}

fn parse(json: &str) -> Result<Quality> {
    let out: Output = serde_json::from_str(json).context("parse ffprobe output")?;
    let s = out
        .streams
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("no video stream"))?;
    let bitrate = s
        .bit_rate
        .or(out.format.and_then(|f| f.bit_rate))
        .and_then(|b| b.parse::<f64>().ok())
        // HLS reports nonsense like "9" for format bit_rate; real video is well above 10 kb/s.
        .filter(|b| *b >= 10_000.0);
    Ok(Quality {
        width: s.width,
        height: s.height,
        fps: rate(s.avg_frame_rate.as_deref()).or(rate(s.r_frame_rate.as_deref())),
        video_codec: s.codec_name,
        bitrate,
        startup_ms: None,
        fails: 0,
    })
}

#[cfg(test)]
mod tests;
