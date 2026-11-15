//! Mirrors the session and mpv state into `PlayerApi` properties.
use super::super::state::SlintApp;
use super::super::{AppWindow, PlayerApi, TrackItem};
use super::chrome;
use crate::model::MediaKind;
use crate::player::{TrackInfo, TrackKind};
use crate::ui::player::controls::fmt_time;
use slint::{ComponentHandle, ModelRc, SharedString};
use std::time::Instant;

pub fn init_models(a: &SlintApp, win: &AppWindow) {
    let api = win.global::<PlayerApi>();
    api.set_audio_tracks(ModelRc::from(a.player.audio.clone()));
    api.set_sub_tracks(ModelRc::from(a.player.subs.clone()));
    api.set_zap_items(ModelRc::from(a.player.zap_items.clone()));
}

fn items(tracks: &[TrackInfo], kind: TrackKind, off: bool) -> Vec<TrackItem> {
    let of_kind: Vec<_> = tracks.iter().filter(|t| t.kind == kind).collect();
    let mut out = Vec::new();
    if off && !of_kind.is_empty() {
        out.push(TrackItem {
            id: -1,
            label: "Off".into(),
            selected: !of_kind.iter().any(|t| t.selected),
        });
    }
    out.extend(of_kind.iter().map(|t| TrackItem {
        id: t.id as i32,
        label: t.label().into(),
        selected: t.selected,
    }));
    out
}

fn set_model(model: &slint::VecModel<TrackItem>, rows: Vec<TrackItem>) {
    model.set_vec(rows);
}

fn meta_line(media: MediaKind, duration: Option<f64>) -> String {
    let kind = match media {
        MediaKind::Live => "Live channel",
        MediaKind::Movie => "Movie",
        MediaKind::Series => "Episode",
    };
    match duration {
        Some(d) if media != MediaKind::Live => format!("{kind} · {}", fmt_time(d)),
        _ => kind.to_string(),
    }
}

/// Pushes everything the pane shows; cheap enough to run on every mpv wakeup and timer tick.
pub fn push(a: &mut SlintApp) {
    let Some(w) = a.win.upgrade() else {
        return;
    };
    let api = w.global::<PlayerApi>();
    let ar = &mut a.player;
    let (Some(core), Some(s)) = (ar.core.as_ref(), ar.session.as_ref()) else {
        api.set_active(false);
        api.set_fullscreen(false);
        api.set_mini(false);
        api.set_pip(false);
        api.set_next_label(SharedString::new());
        api.set_zap_text(SharedString::new());
        return;
    };
    let st = core.state();
    api.set_active(true);
    api.set_fullscreen(ar.fullscreen);
    api.set_mini(ar.mini);
    api.set_pip(ar.mini);
    api.set_paused(st.paused);
    api.set_buffering(st.buffering || s.retry_at.is_some() || st.time_pos.is_none());
    api.set_is_live(s.is_live());
    api.set_muted(st.muted);
    api.set_volume(st.volume as f32);
    api.set_position(st.time_pos.unwrap_or(0.0) as f32);
    api.set_duration(st.duration.unwrap_or(0.0) as f32);
    api.set_pos_text(fmt_time(st.time_pos.unwrap_or(0.0)).into());
    api.set_dur_text(st.duration.map(fmt_time).unwrap_or_default().into());
    api.set_tech(chrome::tech_line(core).into());
    api.set_behind(chrome::behind_s(core));
    api.set_tag(if s.is_live() { "LIVE" } else { "VOD" }.into());
    api.set_title(s.req.title.as_str().into());
    api.set_meta(meta_line(s.req.media, st.duration).into());
    api.set_error(s.error.clone().unwrap_or_default().into());
    api.set_zap_text(ar.zap.text().unwrap_or_default().into());
    let (label, secs) = s.autonext.as_ref().map_or((String::new(), 0), |n| {
        let left = n.deadline.saturating_duration_since(Instant::now());
        (n.label.clone(), left.as_secs() as i32 + 1)
    });
    api.set_next_label(label.into());
    api.set_next_secs(secs);
    if st.tracks != ar.shown_tracks {
        set_model(&ar.audio, items(&st.tracks, TrackKind::Audio, false));
        set_model(&ar.subs, items(&st.tracks, TrackKind::Sub, true));
        ar.shown_tracks = st.tracks.clone();
    }
    if api.get_show_stats() {
        let lines = core.stats().lines();
        let join = |f: fn(&(&'static str, String)) -> String| {
            lines.iter().map(f).collect::<Vec<_>>().join("\n")
        };
        api.set_stats_labels(join(|l| l.0.to_string()).into());
        api.set_stats_values(join(|l| l.1.clone()).into());
    }
}
