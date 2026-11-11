use super::{playlist, setup};
use crate::db::progress::{Watch, classify, fraction};
use crate::model::{MediaKind, ParsedPlaylist};

#[test]
fn classification_thresholds() {
    assert_eq!(fraction(Some(5.0), Some(0.0)), None);
    assert_eq!(fraction(None, Some(10.0)), None);
    assert_eq!(classify(None), Watch::Unstarted);
    assert_eq!(classify(fraction(Some(1.0), Some(100.0))), Watch::Unstarted);
    assert_eq!(classify(fraction(Some(3.0), Some(100.0))), Watch::Partial);
    assert_eq!(classify(fraction(Some(91.0), Some(100.0))), Watch::Partial);
    assert_eq!(classify(fraction(Some(92.0), Some(100.0))), Watch::Watched);
    assert_eq!(classify(fraction(Some(150.0), Some(100.0))), Watch::Watched);
}

fn movies() -> ParsedPlaylist {
    let mut pl = playlist(&["c"], &[("a", "c", "A"), ("b", "c", "B"), ("c", "c", "C")]);
    for s in &mut pl.streams {
        s.media = MediaKind::Movie;
    }
    pl.categories[0].media = MediaKind::Movie;
    pl
}

#[test]
fn continue_lists_partial_movies_newest_first() {
    let (mut db, p, s) = setup();
    db.replace_source_data(s, &movies()).unwrap();
    let m = MediaKind::Movie;
    db.save_position(p, s, m, "a", 500.0, 1000.0).unwrap();
    db.save_position(p, s, m, "b", 990.0, 1000.0).unwrap();
    db.save_position(p, s, m, "c", 300.0, 1000.0).unwrap();
    let keys = |db: &crate::db::Db| -> Vec<String> {
        db.continue_watching(p, m, 10)
            .unwrap()
            .into_iter()
            .map(|r| r.key)
            .collect()
    };
    let got = keys(&db);
    assert_eq!(got.len(), 2);
    assert!(got.contains(&"a".to_string()) && got.contains(&"c".to_string()));
    db.set_watched(p, s, m, "a", true).unwrap();
    assert_eq!(keys(&db), ["c"]);
    db.set_watched(p, s, m, "a", false).unwrap();
    assert_eq!(db.continue_watching(p, m, 10).unwrap().len(), 1);
}

#[test]
fn series_continue_is_one_row_per_series_with_episode() {
    let (mut db, p, s) = setup();
    let mut pl = playlist(&["c"], &[("sr", "c", "Show")]);
    pl.streams[0].media = MediaKind::Series;
    pl.categories[0].media = MediaKind::Series;
    db.replace_source_data(s, &pl).unwrap();
    let m = MediaKind::Series;
    db.record_episode_watch(p, s, "ep:1", "S1E1 · One", "sr")
        .unwrap();
    db.save_position(p, s, m, "ep:1", 400.0, 1000.0).unwrap();
    db.conn
        .execute(
            "UPDATE history SET watched_at = 1 WHERE stream_key = 'ep:1'",
            [],
        )
        .unwrap();
    db.record_episode_watch(p, s, "ep:2", "S1E2 · Two", "sr")
        .unwrap();
    db.save_position(p, s, m, "ep:2", 100.0, 1000.0).unwrap();
    let rows = db.continue_watching(p, m, 10).unwrap();
    assert_eq!(rows.len(), 1);
    let ep = rows[0].resume_episode.as_ref().unwrap();
    assert_eq!((ep.id.as_str(), ep.label.as_str()), ("2", "S1E2 · Two"));
    assert_eq!(db.episode_progress(p, s).unwrap().len(), 2);
    db.set_watched(p, s, m, "ep:2", true).unwrap();
    assert!(db.continue_watching(p, m, 10).unwrap().is_empty());
}
