use super::setup;
use crate::db::track_prefs::{LangKind, TrackLangs};
use crate::model::MediaKind;

#[test]
fn langs_roundtrip_without_touching_favorites() {
    let (db, p, s) = setup();
    let m = MediaKind::Live;
    assert_eq!(db.track_langs(p, s, m, "k").unwrap(), TrackLangs::default());
    db.set_favorite(p, s, m, "k", true).unwrap();
    db.set_track_lang(p, s, m, "k", LangKind::Audio, "fra")
        .unwrap();
    db.set_track_lang(p, s, m, "k", LangKind::Sub, "off")
        .unwrap();
    let langs = db.track_langs(p, s, m, "k").unwrap();
    assert_eq!(langs.audio.as_deref(), Some("fra"));
    assert_eq!(langs.sub.as_deref(), Some("off"));
    db.set_track_lang(p, s, m, "k", LangKind::Sub, "eng")
        .unwrap();
    assert_eq!(
        db.track_langs(p, s, m, "k").unwrap().sub.as_deref(),
        Some("eng")
    );
    assert_eq!(db.favorites(p, m).unwrap().len(), 1);
}
