use super::{http, m3u, xtream};
use crate::model::{Source, SourceKind};
use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufReader, Read};

const SAMPLE_BYTES: u64 = 64 * 1024;

/// One-line verdict for the "Test" button. Blocking; call from the worker thread.
pub fn test_connection(source: &Source) -> Result<String> {
    match source.kind {
        SourceKind::Xtream => {
            let json = xtream::check_account(source)?;
            let info = xtream::AccountInfo::parse(&json).context("unexpected account response")?;
            Ok(format!("OK: {}", info.summary()))
        }
        SourceKind::M3uUrl => {
            let client = http::client(source.user_agent.as_deref())?;
            describe(http::SafeRead(http::get(&client, &source.url)?))
        }
        SourceKind::M3uFile => describe(File::open(&source.url).context("open playlist file")?),
    }
}

fn describe(reader: impl Read) -> Result<String> {
    let mut sample = Vec::new();
    reader
        .take(SAMPLE_BYTES)
        .read_to_end(&mut sample)
        .context("read playlist")?;
    let (playlist, is_hls) = m3u::parse_detect(BufReader::new(&sample[..]))?;
    // the parser accepts any text line as an entry, so require real playlist markup
    let marked = String::from_utf8_lossy(&sample).contains("#EXTINF");
    Ok(if is_hls {
        "Single HLS stream".into()
    } else if !marked || playlist.streams.is_empty() {
        "Not a playlist".into()
    } else {
        format!(
            "M3U playlist (≈{} entries in first 64KB)",
            playlist.streams.len()
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_samples() {
        let m3u = include_bytes!("../../tests/fixtures/sources/messy.m3u");
        assert!(describe(&m3u[..]).unwrap().starts_with("M3U playlist (≈"));
        let hls = include_bytes!("../../tests/fixtures/sources/hls_master.m3u8");
        assert_eq!(describe(&hls[..]).unwrap(), "Single HLS stream");
        assert_eq!(
            describe(&b"<html>nope</html>"[..]).unwrap(),
            "Not a playlist"
        );
    }
}
