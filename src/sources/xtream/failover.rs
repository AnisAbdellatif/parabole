use crate::model::Source;
use anyhow::{Result, anyhow};
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::Arc;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;
use url::Url;

const PROBE_TIMEOUT: Duration = Duration::from_secs(4);

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

/// Runs `f` against the first reachable URL (preference order), then the others, until one succeeds.
pub fn run<T>(source: &Source, f: impl Fn(&Source) -> Result<T>) -> Result<T> {
    let mut all = candidates(source);
    if all.len() > 1 {
        let first = pick_index(all.len(), {
            let urls: Vec<String> = all.iter().map(|s| s.url.clone()).collect();
            Arc::new(move |i| reachable(&urls[i]))
        });
        let healthy = all.remove(first);
        all.insert(0, healthy);
    }
    let mut failures = Vec::new();
    for s in &all {
        match f(s) {
            Ok(v) => return Ok(v),
            Err(e) if all.len() == 1 => return Err(e),
            Err(e) => failures.push(format!("{}: {e:#}", host(&s.url))),
        }
    }
    let shown = failures
        .iter()
        .take(3)
        .cloned()
        .collect::<Vec<_>>()
        .join("; ");
    let more = failures.len().saturating_sub(3);
    let more = if more > 0 {
        format!(" (+{more} more)")
    } else {
        String::new()
    };
    Err(anyhow!(
        "all {} server URLs failed - {shown}{more}",
        all.len()
    ))
}

fn host(base: &str) -> String {
    Url::parse(base)
        .ok()
        .and_then(|u| u.host_str().map(str::to_string))
        .unwrap_or_else(|| "?".into())
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

/// Index of the most preferred healthy candidate, probing all concurrently (0 if none is healthy).
/// Returns as soon as a healthy one is known and every more preferred one has failed.
fn pick_index(n: usize, probe: Arc<dyn Fn(usize) -> bool + Send + Sync>) -> usize {
    let (tx, rx) = channel();
    for i in 0..n {
        let (tx, probe) = (tx.clone(), probe.clone());
        thread::spawn(move || {
            let _ = tx.send((i, probe(i)));
        });
    }
    drop(tx);
    let mut done: Vec<Option<bool>> = vec![None; n];
    for (i, ok) in rx {
        done[i] = Some(ok);
        let Some(best) = done.iter().position(|d| *d != Some(false)) else {
            break;
        };
        if done[best] == Some(true) {
            return best;
        }
    }
    0
}

/// First URL (primary, then mirrors) accepting TCP connections; the primary if none do.
pub fn pick(source: &Source) -> Source {
    let mut all = candidates(source);
    if all.len() == 1 {
        return all.remove(0);
    }
    let urls: Vec<String> = all.iter().map(|s| s.url.clone()).collect();
    let first = pick_index(all.len(), Arc::new(move |i| reachable(&urls[i])));
    all.swap_remove(first)
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
        let msg = format!("{err:#}");
        assert!(msg.contains("all 2 server URLs failed") && msg.contains("a: down"));
    }

    fn fake(
        healthy: &'static [bool],
        delay_ms: &'static [u64],
    ) -> Arc<dyn Fn(usize) -> bool + Send + Sync> {
        Arc::new(move |i| {
            thread::sleep(Duration::from_millis(delay_ms[i]));
            healthy[i]
        })
    }

    #[test]
    fn pick_index_prefers_configured_order_over_speed() {
        let i = pick_index(3, fake(&[true, true, true], &[60, 0, 0]));
        assert_eq!(i, 0);
        let i = pick_index(3, fake(&[false, true, true], &[0, 40, 0]));
        assert_eq!(i, 1);
    }

    #[test]
    fn pick_index_does_not_wait_for_slow_less_preferred() {
        let t = std::time::Instant::now();
        let i = pick_index(2, fake(&[true, false], &[0, 2000]));
        assert_eq!(i, 0);
        assert!(t.elapsed() < Duration::from_millis(1000));
    }

    #[test]
    fn pick_index_defaults_to_primary_when_all_dead() {
        assert_eq!(pick_index(2, fake(&[false, false], &[0, 0])), 0);
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
