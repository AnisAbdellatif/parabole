use super::*;
use chrono::{TimeZone, Utc};

fn tz() -> FixedOffset {
    FixedOffset::east_opt(3600).unwrap()
}

fn ts(d: u32, h: u32, m: u32) -> i64 {
    Utc.with_ymd_and_hms(2026, 11, d, h, m, 0)
        .unwrap()
        .timestamp()
}

fn now() -> i64 {
    ts(16, 12, 0)
}

fn bc(channel: &str, country: &str, lang: Option<&str>) -> Broadcast {
    Broadcast {
        channel: channel.into(),
        country: country.into(),
        lang: lang.map(Into::into),
        provider: "p".into(),
    }
}

fn game(key: &str, kickoff: i64, status: MatchStatus, b: Vec<Broadcast>) -> Match {
    Match {
        key: key.into(),
        competition: "Nations League".into(),
        home: "Germany".into(),
        away: "Netherlands".into(),
        kickoff_utc: kickoff,
        status,
        broadcasts: b,
    }
}

fn strs(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

#[test]
fn sections_only_on_group_change() {
    let live = MatchStatus::Live {
        clock: "67'".into(),
        home: 1,
        away: 2,
    };
    let ms = vec![
        game("a", ts(16, 11, 0), live, vec![]),
        game("b", ts(16, 19, 45), MatchStatus::Upcoming, vec![]),
        game("c", ts(16, 20, 0), MatchStatus::Upcoming, vec![]),
        game("d", ts(17, 12, 0), MatchStatus::Upcoming, vec![]),
        game("e", ts(18, 12, 0), MatchStatus::Upcoming, vec![]),
    ];
    let r = rows(&ms, &[], now(), tz());
    let sections: Vec<&str> = r.iter().map(|x| x.section.as_str()).collect();
    assert_eq!(sections, ["LIVE", "TODAY", "", "TOMORROW", "WED 18 NOV"]);
    assert_eq!((r[0].time.as_str(), r[0].score.as_str()), ("67'", "1–2"));
    assert!(r[0].live);
    assert_eq!((r[1].time.as_str(), r[1].score.as_str()), ("20:45", ""));
    assert_eq!(r[1].teams, "Germany – Netherlands");
}

#[test]
fn local_midnight_boundary() {
    let ms = vec![game("a", ts(16, 23, 30), MatchStatus::Upcoming, vec![])];
    let r = rows(&ms, &[], now(), tz());
    assert_eq!(r[0].section, "TOMORROW");
    assert_eq!(r[0].time, "00:30");
}

#[test]
fn chips_order_and_overflow() {
    let b: Vec<Broadcast> = ["de", "en", "fr", "ar", "es", "it"]
        .iter()
        .map(|l| bc("X", "ZZ", Some(l)))
        .collect();
    assert_eq!(chips(&b, &strs(&["fr", "xx", "en"])), "FR EN AR DE +2");
    assert_eq!(chips(&[bc("X", "ZZ", None)], &[]), "");
    assert_eq!(chips(&b[..1], &[]), "DE");
}

#[test]
fn details_group_order_and_dedupe() {
    let b = vec![
        bc("beIN Sport 1 HD", "SA", Some("ar")),
        bc("ARD Das Erste", "DE", Some("de")),
        bc("Fox Sports 1", "US", Some("en")),
        bc("Mystery", "CH", None),
        bc("fox sports 1", "US", Some("en")),
        bc("Amazon Prime PPV", "GB", Some("en")),
    ];
    let m = game("a", ts(16, 19, 45), MatchStatus::Upcoming, b);
    let (title, sub, ch) = details(&m, &strs(&["en"]), tz());
    assert_eq!(title, "Germany – Netherlands");
    assert_eq!(sub, "Nations League · Mon 16 Nov, 20:45");
    let texts: Vec<&str> = ch.iter().map(|c| c.text.as_str()).collect();
    assert_eq!(
        texts,
        [
            "English",
            "Amazon Prime PPV (GB)",
            "Fox Sports 1 (US)",
            "Arabic",
            "beIN Sport 1 HD (SA)",
            "German",
            "ARD Das Erste (DE)",
            "Other / multilingual",
            "Mystery (CH)"
        ]
    );
    assert!(ch[0].header && !ch[1].header);
    assert_eq!(ch[1].channel, "Amazon Prime PPV");
}

#[test]
fn details_sub_for_live_and_finished() {
    let live = MatchStatus::Live {
        clock: "HT".into(),
        home: 0,
        away: 0,
    };
    let (_, sub, _) = details(&game("a", 0, live, vec![]), &[], tz());
    assert_eq!(sub, "Nations League · Live HT · 0–0");
    let (_, sub, _) = details(&game("a", 0, MatchStatus::Finished, vec![]), &[], tz());
    assert_eq!(sub, "Nations League · Finished");
}

#[test]
fn status_text() {
    let p = vec![
        ProviderStatus {
            name: "365scores".into(),
            ok: true,
            error: None,
        },
        ProviderStatus {
            name: "LiveScore".into(),
            ok: false,
            error: Some("x".into()),
        },
    ];
    let t = ts(16, 13, 32);
    assert_eq!(
        status(&p, false, Some(t), tz()),
        "Updated 14:32 · LiveScore unavailable"
    );
    assert_eq!(
        status(&p, true, Some(t), tz()),
        "Updating… · LiveScore unavailable"
    );
    assert_eq!(status(&[], false, None, tz()), "");
}

#[test]
fn details_merge_same_channel_across_countries() {
    let m = game(
        "s365:1",
        now(),
        MatchStatus::Upcoming,
        vec![
            bc("beIN Sport 1 HD", "SA", Some("ar")),
            bc("beIN Sport 1 HD", "EG", Some("ar")),
            bc("ON Time Sports", "EG", Some("ar")),
        ],
    );
    let (_, _, ch) = details(&m, &strs(&["ar"]), tz());
    let text: Vec<&str> = ch.iter().map(|c| c.text.as_str()).collect();
    assert_eq!(
        text,
        ["Arabic", "beIN Sport 1 HD (EG, SA)", "ON Time Sports (EG)"]
    );
    assert_eq!(ch[1].channel, "beIN Sport 1 HD");
}
