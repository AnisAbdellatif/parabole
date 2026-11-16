use chrono::{TimeZone, Utc};
use flate2::{Compression, write::GzEncoder};
use parabole::db::Db;
use parabole::epg::{Window, fetch_all};
use parabole::model::*;
use parabole::sources::guide::guide_urls;
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
#[ignore = "epg2: rewritten with the worker"]
fn fetches_parses_stores_and_answers_now_next() {
    let now = Utc::now().timestamp();
    let xml = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><tv>{}{}{}{}</tv>",
        prog("tf1.fr", now - 1800, now + 1800, "Journal"),
        prog("tf1.fr", now + 1800, now + 5400, "Météo &amp; Sport"),
        prog("arte.fr", now - 4 * 3600, now - 3 * 3600, "Old"),
        prog("zzz", now - 600, now + 600, "Unknown"),
    );
    let mut gz = GzEncoder::new(Vec::new(), Compression::default());
    gz.write_all(xml.as_bytes()).unwrap();
    let url = serve(gz.finish().unwrap());

    let mut db = Db::open_in_memory().unwrap();
    let profile = db.create_profile("p").unwrap();
    let src = source();
    let sid = db.add_source(profile, &src).unwrap();
    db.replace_source_data(
        sid,
        &ParsedPlaylist {
            categories: vec![ParsedCategory {
                media: MediaKind::Live,
                ext_key: "c".into(),
                name: "C".into(),
            }],
            streams: vec![
                live("1", "TF1", "TF1.fr ", MediaKind::Live),
                live("2", "Arte", "arte.fr", MediaKind::Live),
                live("3", "Movie", "tf1.fr", MediaKind::Movie),
            ],
            epg_url: Some(url),
            account_json: None,
        },
    )
    .unwrap();
    let ids = |name: &str| {
        db.streams_in_category(profile, sid, MediaKind::Live, "c", true)
            .unwrap()
            .into_iter()
            .find(|s| s.name == name)
            .unwrap()
            .id
    };
    let (tf1, arte) = (ids("TF1"), ids("Arte"));

    let epg_url = db.source_epg_url(sid).unwrap();
    let source = db.source(sid).unwrap();
    let urls = guide_urls(&source, epg_url.as_deref()).unwrap();
    assert_eq!(urls.len(), 1);
    // TODO(epg2): rewrite with the worker
    let channels: std::collections::HashSet<String> = Default::default();
    let progs = fetch_all(&urls, None, Window::around(now), &mut |_| channels.clone())
        .unwrap()
        .programmes;
    let g = db.provider_guide(sid).unwrap().unwrap();
    db.replace_guide(g, &[], &progs, now).unwrap();

    let nn = db.epg_now_next(&[tf1, arte], now).unwrap();
    let t = &nn[&tf1];
    assert_eq!(t.now.title, "Journal");
    assert_eq!(t.next.as_ref().unwrap().title, "Météo & Sport");
    assert!(!nn.contains_key(&arte));
    assert!(progs.iter().all(|p| p.channel != "zzz"));
    let sched = db.epg_schedule(tf1, now - 7200, now + 86400).unwrap();
    let titles: Vec<_> = sched.iter().map(|p| p.title.as_str()).collect();
    assert_eq!(titles, ["Journal", "Météo & Sport"]);
    assert!(db.epg_schedule(arte, 0, i64::MAX).unwrap().is_empty());
}

#[test]
#[ignore]
fn real_guide_smoke() {
    let Ok(path) = std::env::var("PARABOLE_DB") else {
        eprintln!("PARABOLE_DB unset; skipping");
        return;
    };
    let db = Db::open(std::path::Path::new(&path)).unwrap();
    let now = Utc::now().timestamp();
    let mut ids = Vec::new();
    for p in db.profiles().unwrap() {
        for s in db.profile_sources(p.id).unwrap() {
            let mut n = 0;
            for c in db
                .categories(p.id, Some(s.id), MediaKind::Live, true)
                .unwrap()
            {
                for st in db
                    .streams_in_category(p.id, s.id, MediaKind::Live, &c.key, true)
                    .unwrap()
                {
                    n += db
                        .epg_schedule(st.id, now - 7200, now + 3 * 86400)
                        .unwrap()
                        .len();
                    ids.push((st.id, st.name));
                }
            }
            println!(
                "source {} ({}): {n} programmes in window (summed per stream)",
                s.id, s.name
            );
        }
    }
    ids.sort();
    ids.dedup_by_key(|x| x.0);
    let only: Vec<i64> = ids.iter().map(|x| x.0).collect();
    let nn = db.epg_now_next(&only, now).unwrap();
    println!("live streams: {}, airing now: {}", ids.len(), nn.len());
    let hm = |t: i64| Utc.timestamp_opt(t, 0).unwrap().format("%H:%M").to_string();
    for (id, name) in ids.iter().filter(|(id, _)| nn.contains_key(id)).take(5) {
        let x = &nn[id];
        println!(
            "{name}: {} {}-{} → {}",
            x.now.title,
            hm(x.now.start),
            hm(x.now.stop),
            x.next.as_ref().map_or("-", |n| n.title.as_str())
        );
    }
    assert!(!nn.is_empty());
}
