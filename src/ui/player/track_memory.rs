//! Remembers the audio/subtitle language picked for a stream and restores it on the next load.
use super::session::Session;
use crate::db::Db;
use crate::db::track_prefs::{LangKind, TrackLangs};
use crate::player::embed::EmbeddedPlayer;
use crate::player::{TrackInfo, TrackKind};
use std::time::{Duration, Instant};

/// mpv needs a moment to reflect our own track selection; changes seen before then are not the user's.
const SETTLE: Duration = Duration::from_millis(1500);
const OFF: &str = "off";

#[derive(Default)]
pub struct TrackMemory {
    applied: bool,
    settle_until: Option<Instant>,
    armed: bool,
    audio: Option<i64>,
    sub: Option<i64>,
}

/// Which track to select to honour `pref`: None = leave as is, Some(None) = subtitles off.
fn choose(tracks: &[TrackInfo], kind: TrackKind, pref: Option<&str>) -> Option<Option<i64>> {
    let pref = pref?;
    let of_kind = || tracks.iter().filter(|t| t.kind == kind);
    if pref == OFF {
        return of_kind().any(|t| t.selected).then_some(None);
    }
    let wanted = of_kind().find(|t| {
        t.lang
            .as_deref()
            .is_some_and(|l| l.eq_ignore_ascii_case(pref))
    })?;
    (!wanted.selected).then_some(Some(wanted.id))
}

fn selected(tracks: &[TrackInfo], kind: TrackKind) -> Option<&TrackInfo> {
    tracks.iter().find(|t| t.kind == kind && t.selected)
}

pub fn tick(db: &Db, player: &mut EmbeddedPlayer, s: &mut Session, ctx: &egui::Context) {
    let tracks = player.state().tracks.clone();
    if tracks.is_empty() {
        s.memory = TrackMemory::default();
        return;
    }
    let Some(track) = s.track.clone() else {
        return;
    };
    let m = &mut s.memory;
    if !m.applied {
        let langs = db
            .track_langs(track.profile, track.source, track.media, &track.key)
            .unwrap_or_default();
        apply(player, &tracks, &langs);
        m.applied = true;
        m.settle_until = Some(Instant::now() + SETTLE);
        ctx.request_repaint_after(SETTLE + Duration::from_millis(100));
        return;
    }
    if !m.armed {
        if m.settle_until.is_some_and(|t| Instant::now() < t) {
            return;
        }
        m.audio = selected(&tracks, TrackKind::Audio).map(|t| t.id);
        m.sub = selected(&tracks, TrackKind::Sub).map(|t| t.id);
        m.armed = true;
        return;
    }
    let audio = selected(&tracks, TrackKind::Audio);
    if audio.map(|t| t.id) != m.audio {
        m.audio = audio.map(|t| t.id);
        if let Some(lang) = audio.and_then(|t| t.lang.as_deref()) {
            let _ = db.set_track_lang(
                track.profile,
                track.source,
                track.media,
                &track.key,
                LangKind::Audio,
                lang,
            );
        }
    }
    let sub = selected(&tracks, TrackKind::Sub);
    if sub.map(|t| t.id) != m.sub {
        m.sub = sub.map(|t| t.id);
        let lang = match sub {
            None => Some(OFF),
            Some(t) => t.lang.as_deref(),
        };
        if let Some(lang) = lang {
            let _ = db.set_track_lang(
                track.profile,
                track.source,
                track.media,
                &track.key,
                LangKind::Sub,
                lang,
            );
        }
    }
}

fn apply(player: &EmbeddedPlayer, tracks: &[TrackInfo], langs: &TrackLangs) {
    if let Some(id) = choose(tracks, TrackKind::Audio, langs.audio.as_deref()) {
        player.select_track(TrackKind::Audio, id);
    }
    if let Some(id) = choose(tracks, TrackKind::Sub, langs.sub.as_deref()) {
        player.select_track(TrackKind::Sub, id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(id: i64, kind: TrackKind, lang: &str, selected: bool) -> TrackInfo {
        TrackInfo {
            id,
            kind,
            title: None,
            lang: Some(lang.into()),
            selected,
        }
    }

    #[test]
    fn matches_language_case_insensitively() {
        let tracks = [
            t(1, TrackKind::Audio, "eng", true),
            t(2, TrackKind::Audio, "fra", false),
        ];
        let pick = |pref| choose(&tracks, TrackKind::Audio, pref);
        assert_eq!(pick(Some("FRA")), Some(Some(2)));
        assert_eq!(pick(Some("eng")), None);
        assert_eq!(pick(Some("deu")), None);
        assert_eq!(pick(None), None);
    }

    #[test]
    fn off_only_acts_when_a_subtitle_is_on() {
        let on = [t(3, TrackKind::Sub, "eng", true)];
        let off = [t(3, TrackKind::Sub, "eng", false)];
        assert_eq!(choose(&on, TrackKind::Sub, Some("off")), Some(None));
        assert_eq!(choose(&off, TrackKind::Sub, Some("off")), None);
        assert_eq!(choose(&off, TrackKind::Sub, Some("eng")), Some(Some(3)));
    }
}
