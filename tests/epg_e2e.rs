use chrono::{TimeZone, Utc};
use flate2::{Compression, write::GzEncoder};
use parabole::db::Db;
use parabole::epg::matching;
use parabole::epg::{Window, fetch_all};
use parabole::model::*;
use parabole::worker::epg::relink;
use std::io::{Read, Write};
use std::net::TcpListener;

fn ts(t: i64) -> String {
    Utc.timestamp_opt(t, 0)
        .unwrap()
        .format("%Y%m%d%H%M%S +0000")
        .to_string()
}

fn prog(ch: &str, a: i64, b: i64, title: &str) -> String {
    format!(
        "<programme start=\"{}\" stop=\"{}\" channel=\"{ch}\"><title>{title}</title></programme>",
        ts(a),
        ts(b)
    )
}

fn serve(body: Vec<u8>) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    std::thread::spawn(move || {
        for conn in listener.incoming() {
            let Ok(mut c) = conn else { continue };
            let mut buf = [0u8; 4096];
            let _ = c.read(&mut buf);
            let head = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/gzip\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            let _ = c.write_all(head.as_bytes());
            let _ = c.write_all(&body);
        }
    });
    format!("http://{addr}/guide.xml.gz")
}

fn live(key: &str, name: &str, epg: &str, media: MediaKind) -> ParsedStream {
    ParsedStream {
        media,
        ext_key: key.into(),
        category_key: Some("c".into()),
        name: name.into(),
        epg_id: Some(epg.into()),
        ..Default::default()
    }
}

fn source() -> Source {
    Source {
        id: 0,
        name: "S".into(),
        kind: SourceKind::M3uUrl,
        url: "http://example.test/list.m3u".into(),
        username: None,
        password: None,
        user_agent: None,
        mirrors: Vec::new(),
        output_format: None,
        last_refresh: None,
        last_error: None,
    }
}

#[test]
fn fetches_parses_stores_and_answers_now_next() {
    let now = Utc::now().timestamp();
    let ch = |id: &str, n: &str| {
        format!("<channel id=\"{id}\"><display-name>{n}</display-name></channel>")
    };
    let xml = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><tv>{}{}{}{}{}</tv>",
        ch("HannibalTV.tn@SD", "Hannibal TV"),
        ch("Tunisia.Nat.1.ae", "Tunisia Nat 1"),
        prog("HannibalTV.tn@SD", now - 1800, now + 1800, "Journal"),
        prog(
            "HannibalTV.tn@SD",
            now + 1800,
            now + 5400,
            "Météo &amp; Sport"
        ),
        prog("Tunisia.Nat.1.ae", now - 600, now + 600, "Watania"),
    );
    let mut gz = GzEncoder::new(Vec::new(), Compression::default());
    gz.write_all(xml.as_bytes()).unwrap();
    let url = serve(gz.finish().unwrap());
    let url_host = url
        .trim_start_matches("http://")
        .trim_end_matches("/guide.xml.gz")
        .to_string();

    let mut db = Db::open_in_memory().unwrap();
    let profile = db.create_profile("p").unwrap();
    let sid = db.add_source(profile, &source()).unwrap();
    db.replace_source_data(
        sid,
        &ParsedPlaylist {
            categories: vec![ParsedCategory {
                media: MediaKind::Live,
                ext_key: "c".into(),
                name: "C".into(),
            }],
            streams: vec![
                live("1", "AR: Hannibal ◉", "hannibaltv.tn", MediaKind::Live),
                live("2", "AR: Nessma", "ts", MediaKind::Live),
                live("3", "Movie", "hannibaltv.tn", MediaKind::Movie),
            ],
            epg_url: None,
            account_json: None,
        },
    )
    .unwrap();
    let id = |name: &str| {
        db.streams_in_category(profile, sid, MediaKind::Live, "c", true)
            .unwrap()
            .into_iter()
            .find(|s| s.name == name)
            .unwrap()
            .id
    };
    let (hannibal, nessma) = (id("AR: Hannibal ◉"), id("AR: Nessma"));

    let provider = db.provider_guide(sid).unwrap().unwrap();
    assert!(db.source_epg_url(sid).unwrap().is_none());
    let user = db.add_epg_guide(&url, "Extra").unwrap();
    let streams = db.epg_stream_keys().unwrap();
    assert_eq!(streams.len(), 2, "movie ignored");
    let g = fetch_all(&[url], None, Window::around(now), &mut |c| {
        matching::wanted(&streams, c)
    })
    .unwrap();
    db.replace_guide(user, &g.channels, &g.programmes, now)
        .unwrap();
    relink(&mut db).unwrap();

    let nn = db.epg_now_next(&[hannibal, nessma], now).unwrap();
    let t = &nn[&hannibal];
    assert_eq!(t.now.title, "Journal");
    assert_eq!(t.next.as_ref().unwrap().title, "Météo & Sport");
    assert!(!nn.contains_key(&nessma));
    let cov = db.epg_coverage().unwrap();
    assert_eq!(cov.get(&user), Some(&1));
    assert!(!cov.contains_key(&provider));
    let sched = db.epg_schedule(hannibal, now - 7200, now + 86400).unwrap();
    let titles: Vec<_> = sched.iter().map(|p| p.title.as_str()).collect();
    assert_eq!(titles, ["Journal", "Météo & Sport"]);

    db.set_manual_link(sid, "2", user, "tunisia.nat.1.ae")
        .unwrap();
    // `wanted` ignores manual links, so the worker would not keep this channel's programmes yet
    let url = format!("http://{}/guide.xml.gz", url_host);
    let g = fetch_all(&[url], None, Window::around(now), &mut |c| {
        let mut w = matching::wanted(&streams, c);
        w.insert("tunisia.nat.1.ae".into());
        w
    })
    .unwrap();
    db.replace_guide(user, &g.channels, &g.programmes, now)
        .unwrap();
    relink(&mut db).unwrap();
    for _ in 0..2 {
        let nn = db.epg_now_next(&[nessma], now).unwrap();
        assert_eq!(nn[&nessma].now.title, "Watania");
        relink(&mut db).unwrap();
    }
}

#[test]
#[ignore]
fn real_guide_smoke() {
    let Ok(path) = std::env::var("PARABOLE_DB") else {
        eprintln!("PARABOLE_DB unset; skipping");
        return;
    };
    let mut db = Db::open(std::path::Path::new(&path)).unwrap();
    let now = Utc::now().timestamp();
    if let Ok(url) = std::env::var("PARABOLE_EXTRA_GUIDE") {
        let id = db.add_epg_guide(&url, "Extra").unwrap();
        let streams = db.epg_stream_keys().unwrap();
        let g = fetch_all(&[url], None, Window::around(now), &mut |c| {
            matching::wanted(&streams, c)
        })
        .unwrap();
        db.replace_guide(id, &g.channels, &g.programmes, now)
            .unwrap();
    }
    let t = std::time::Instant::now();
    let n = relink(&mut db).unwrap();
    println!("relink: {n} links in {:?}", t.elapsed());
    let cov = db.epg_coverage().unwrap();
    for g in db.epg_guides().unwrap() {
        println!(
            "guide {}: coverage {}",
            g.name,
            cov.get(&g.id).unwrap_or(&0)
        );
    }
    let keys = [
        "tunisia", "hannibal", "nessma", "hiwar", "watania", "carthage",
    ];
    for p in db.profiles().unwrap() {
        for s in db.profile_sources(p.id).unwrap() {
            let mut rows = Vec::new();
            for c in db
                .categories(p.id, Some(s.id), MediaKind::Live, true)
                .unwrap()
            {
                rows.extend(
                    db.streams_in_category(p.id, s.id, MediaKind::Live, &c.key, true)
                        .unwrap()
                        .into_iter()
                        .filter(|st| keys.iter().any(|k| st.name.to_lowercase().contains(k))),
                );
            }
            let ids: Vec<i64> = rows.iter().map(|r| r.id).collect();
            let nn = db.epg_now_next(&ids, now).unwrap();
            for r in rows {
                let title = nn.get(&r.id).map_or("—", |x| x.now.title.as_str());
                println!("{} → {title}", r.name);
            }
        }
    }
}
