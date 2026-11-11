use crate::model::Source;
use anyhow::{Result, anyhow};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;
use url::Url;

const PROBE_TIMEOUT: Duration = Duration::from_secs(3);

fn candidates(source: &Source) -> Vec<Source> {
    std::iter::once(&source.url)
        .chain(&source.mirrors)
        .map(|base| Source {
            url: base.clone(),
            mirrors: Vec::new(),
            ..source.clone()
        })
        .collect()
}

/// Runs `f` against the primary URL, then each mirror, until one succeeds.
pub fn run<T>(source: &Source, f: impl Fn(&Source) -> Result<T>) -> Result<T> {
    let all = candidates(source);
    let mut last = None;
    for s in &all {
        match f(s) {
            Ok(v) => return Ok(v),
            Err(e) => last = Some(e),
        }
    }
    let last = last.unwrap_or_else(|| anyhow!("no server URL"));
    if all.len() == 1 {
        return Err(last);
    }
    Err(last.context(format!("all {} server URLs failed", all.len())))
}

fn reachable(base: &str) -> bool {
    let Ok(url) = Url::parse(base) else {
        return false;
    };
    let Some(port) = url.port_or_known_default() else {
        return false;
    };
    let Some(host) = url.host_str() else {
        return false;
    };
    (host, port)
        .to_socket_addrs()
        .map(|mut addrs| addrs.any(|a| TcpStream::connect_timeout(&a, PROBE_TIMEOUT).is_ok()))
        .unwrap_or(false)
}

/// First URL (primary, then mirrors) accepting TCP connections; the primary if none do.
pub fn pick(source: &Source) -> Source {
    let mut all = candidates(source);
    if all.len() == 1 {
        return all.remove(0);
    }
    let first_ok = all.iter().position(|s| reachable(&s.url)).unwrap_or(0);
    all.swap_remove(first_ok)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::SourceKind;
    use std::cell::RefCell;
    use std::net::TcpListener;

    fn src(url: &str, mirrors: &[&str]) -> Source {
        Source {
            id: 1,
            name: "s".into(),
            kind: SourceKind::Xtream,
            url: url.into(),
            mirrors: mirrors.iter().map(|m| m.to_string()).collect(),
            username: Some("u".into()),
            password: Some("p".into()),
            user_agent: None,
            output_format: None,
            last_refresh: None,
            last_error: None,
        }
    }

    #[test]
    fn run_falls_through_to_working_mirror() {
        let tried = RefCell::new(Vec::new());
        let out = run(&src("http://a", &["http://b", "http://c"]), |s| {
            tried.borrow_mut().push(s.url.clone());
            if s.url == "http://b" {
                Ok(s.url.clone())
            } else {
                Err(anyhow!("down"))
            }
        });
        assert_eq!(out.unwrap(), "http://b");
        assert_eq!(*tried.borrow(), ["http://a", "http://b"]);
    }

    #[test]
    fn run_reports_when_all_fail() {
        let err = run(&src("http://a", &["http://b"]), |_| -> Result<()> {
            Err(anyhow!("down"))
        })
        .unwrap_err();
        assert!(format!("{err:#}").contains("all 2 server URLs failed"));
    }

    #[test]
    fn pick_skips_dead_primary() {
        let live = TcpListener::bind("127.0.0.1:0").unwrap();
        let dead = TcpListener::bind("127.0.0.1:0").unwrap();
        let dead_url = format!("http://{}", dead.local_addr().unwrap());
        drop(dead);
        let live_url = format!("http://{}", live.local_addr().unwrap());
        let picked = pick(&src(&dead_url, &[&live_url]));
        assert_eq!(picked.url, live_url);
    }
}
