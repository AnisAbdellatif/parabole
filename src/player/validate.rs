use crate::model::{Headers, PlayRequest};
use anyhow::{Result, bail};
use url::Url;

const ALLOWED_SCHEMES: &[&str] = &[
    "http", "https", "rtmp", "rtmps", "rtsp", "udp", "rtp", "mms",
];
const MAX_TITLE: usize = 200;
const MAX_HEADER: usize = 500;

/// Rejects anything a player could interpret as an option, file, or script. Error text never echoes the URL.
pub fn validate_url(raw: &str) -> Result<()> {
    if raw.is_empty() || raw.starts_with('-') {
        bail!("invalid stream URL");
    }
    if raw.chars().any(|c| c.is_whitespace() || c.is_control()) {
        bail!("stream URL contains whitespace or control characters");
    }
    let Ok(url) = Url::parse(raw) else {
        bail!("stream URL is not a valid URL");
    };
    if !ALLOWED_SCHEMES.contains(&url.scheme()) {
        bail!("stream URL scheme '{}' is not allowed", url.scheme());
    }
    Ok(())
}

pub fn sanitize(text: &str, max_chars: usize) -> String {
    text.chars()
        .filter(|c| !c.is_control())
        .take(max_chars)
        .collect()
}

fn clean_opt(value: &Option<String>) -> Option<String> {
    value
        .as_deref()
        .map(|v| sanitize(v, MAX_HEADER))
        .filter(|v| !v.is_empty())
}

/// Validated copy of a request with control characters stripped from title and headers.
pub fn clean_request(req: &PlayRequest) -> Result<PlayRequest> {
    validate_url(&req.url)?;
    Ok(PlayRequest {
        url: req.url.clone(),
        title: sanitize(&req.title, MAX_TITLE),
        headers: Headers {
            user_agent: clean_opt(&req.headers.user_agent),
            referrer: clean_opt(&req.headers.referrer),
            origin: clean_opt(&req.headers.origin),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_streaming_schemes() {
        for u in [
            "http://a/b.m3u8",
            "https://a/b",
            "rtmp://a/b",
            "rtsp://a/b",
            "udp://@239.0.0.1:1234",
            "mms://a/b",
        ] {
            assert!(validate_url(u).is_ok(), "{u}");
        }
    }

    #[test]
    fn rejects_dangerous_input() {
        for u in [
            "",
            "--script=/tmp/x.lua",
            "-v",
            "file:///etc/passwd",
            "javascript:alert(1)",
            "/etc/passwd",
            "http://a/b c",
            "http://a/b\nc",
            "ftp://a/b",
        ] {
            assert!(validate_url(u).is_err(), "{u:?}");
        }
    }

    #[test]
    fn error_does_not_leak_url() {
        let err = validate_url("ftp://user:secret@host/x")
            .unwrap_err()
            .to_string();
        assert!(!err.contains("secret"));
    }

    #[test]
    fn sanitize_strips_controls_and_caps() {
        assert_eq!(sanitize("a\nb\u{0}c\td", 200), "abcd");
        assert_eq!(sanitize(&"x".repeat(500), MAX_TITLE).len(), MAX_TITLE);
    }

    #[test]
    fn clean_request_cleans_headers() {
        let req = PlayRequest {
            url: "http://a/b".into(),
            title: "t\r\nx".into(),
            headers: Headers {
                user_agent: Some("ua\r\nEvil: 1".into()),
                referrer: Some("\n".into()),
                origin: None,
            },
        };
        let c = clean_request(&req).unwrap();
        assert_eq!(c.title, "tx");
        assert_eq!(c.headers.user_agent.as_deref(), Some("uaEvil: 1"));
        assert_eq!(c.headers.referrer, None);
    }
}
