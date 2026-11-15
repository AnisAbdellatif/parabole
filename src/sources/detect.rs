use crate::model::SourceInput;
use anyhow::{Result, bail};
use std::path::Path;
use url::Url;

/// Classify text pasted by the user (PLAN.md §5.4). No network access.
pub fn detect(input: &str) -> Result<SourceInput> {
    let lines: Vec<&str> = input
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    match lines.as_slice() {
        [url, user, pass] if is_http(url) && !user.contains(' ') && !pass.contains(' ') => {
            xtream(url, user, pass)
        }
        [line] => detect_line(line),
        _ => bail!("unrecognized input"),
    }
}

fn detect_line(line: &str) -> Result<SourceInput> {
    if is_http(line) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        return match parts.as_slice() {
            [url] => detect_url(url),
            [url, user, pass] => xtream(url, user, pass),
            _ => bail!("unrecognized input"),
        };
    }
    if let Some(path) = line.strip_prefix("file://") {
        return match Url::parse(line).ok().and_then(|u| u.to_file_path().ok()) {
            Some(p) => Ok(SourceInput::M3uFile(p.to_string_lossy().into_owned())),
            None => Ok(SourceInput::M3uFile(path.to_string())),
        };
    }
    if Path::new(line).exists() {
        return Ok(SourceInput::M3uFile(line.to_string()));
    }
    bail!("not a URL or an existing file")
}

fn detect_url(text: &str) -> Result<SourceInput> {
    let url = parse_http(text)?;
    let is_api = url
        .path_segments()
        .and_then(|mut s| s.next_back())
        .is_some_and(|last| last == "get.php" || last == "player_api.php");
    if is_api {
        let param = |name: &str| {
            url.query_pairs()
                .find(|(k, v)| k == name && !v.is_empty())
                .map(|(_, v)| v.into_owned())
        };
        if let (Some(username), Some(password)) = (param("username"), param("password")) {
            return Ok(SourceInput::Xtream {
                base: base_of(&url),
                username,
                password,
            });
        }
    }
    Ok(SourceInput::M3uUrl(text.to_string()))
}

fn xtream(url: &str, user: &str, pass: &str) -> Result<SourceInput> {
    Ok(SourceInput::Xtream {
        base: base_of(&parse_http(url)?),
        username: user.to_string(),
        password: pass.to_string(),
    })
}

fn is_http(s: &str) -> bool {
    let s = s.to_ascii_lowercase();
    s.starts_with("http://") || s.starts_with("https://")
}

fn parse_http(text: &str) -> Result<Url> {
    let url = Url::parse(text)?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        bail!("not an http(s) URL");
    }
    Ok(url)
}

/// `scheme://host[:port][/prefix]` without a trailing slash or API script name.
fn base_of(url: &Url) -> String {
    let path = url.path().trim_end_matches('/');
    let path = ["/get.php", "/player_api.php"]
        .iter()
        .find_map(|s| path.strip_suffix(s))
        .unwrap_or(path);
    let port = url.port().map(|p| format!(":{p}")).unwrap_or_default();
    format!(
        "{}://{}{port}{path}",
        url.scheme(),
        url.host_str().unwrap_or_default()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn xt(base: &str, u: &str, p: &str) -> SourceInput {
        SourceInput::Xtream {
            base: base.into(),
            username: u.into(),
            password: p.into(),
        }
    }

    #[test]
    fn table() {
        let cases = [
            (
                "http://h.tv:8080/get.php?username=u&password=p&type=m3u_plus",
                xt("http://h.tv:8080", "u", "p"),
            ),
            (
                "https://h.tv/player_api.php?password=p%40x&username=u",
                xt("https://h.tv", "u", "p@x"),
            ),
            (
                "http://h.tv/panel/get.php?username=u&password=p",
                xt("http://h.tv/panel", "u", "p"),
            ),
            (
                "http://h.tv:80/ user pass",
                xt("http://h.tv", "user", "pass"),
            ),
            (
                "  http://h.tv:25461 user pass  ",
                xt("http://h.tv:25461", "user", "pass"),
            ),
            (
                "http://h.tv:25461/\nuser\npass\n",
                xt("http://h.tv:25461", "user", "pass"),
            ),
            (
                "http://h.tv/list.m3u",
                SourceInput::M3uUrl("http://h.tv/list.m3u".into()),
            ),
            (
                "http://h.tv/get.php?type=m3u",
                SourceInput::M3uUrl("http://h.tv/get.php?type=m3u".into()),
            ),
            #[cfg(not(windows))]
            (
                "file:///tmp/a%20b.m3u",
                SourceInput::M3uFile("/tmp/a b.m3u".into()),
            ),
            #[cfg(windows)]
            (
                "file:///C:/tmp/a%20b.m3u",
                SourceInput::M3uFile(r"C:\tmp\a b.m3u".into()),
            ),
            ("Cargo.toml", SourceInput::M3uFile("Cargo.toml".into())),
        ];
        for (input, want) in cases {
            assert_eq!(detect(input).unwrap(), want, "{input}");
        }
    }

    #[test]
    fn rejects_garbage() {
        for input in [
            "",
            "hello",
            "rtmp://h/x",
            "ftp://h/x",
            "http://h one two three",
            "/no/such/file.m3u",
        ] {
            assert!(detect(input).is_err(), "{input}");
        }
    }
}
