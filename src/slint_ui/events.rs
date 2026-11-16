//! Worker events, drained on the UI thread (mirrors `ui/events.rs`).
use super::Core;
use super::state::with_app;
use crate::player::PlayerKind;
use crate::worker::Event;
use slint::ComponentHandle;

pub fn drain() {
    with_app(|a| {
        while let Ok(ev) = a.worker.rx.try_recv() {
            let Some(ev) = super::dialogs::on_event(a, ev) else {
                continue;
            };
            let Some(ev) = super::palette::on_event(a, ev) else {
                continue;
            };
            let Some(ev) = super::player::on_event(a, ev) else {
                continue;
            };
            let Some(ev) = super::browse::on_event(a, ev) else {
                continue;
            };
            let Some(ev) = super::sports::on_event(a, ev) else {
                continue;
            };
            match ev {
                Event::SourceChanged => a.reload_sources(),
                Event::RefreshStarted(id) => {
                    a.refreshing.insert(id);
                    a.push_status();
                }
                Event::RefreshDone(id, res) => {
                    a.refreshing.remove(&id);
                    if let Err(e) = res {
                        a.toast(format!("Refresh failed: {e}"));
                    }
                    a.reload_sources();
                }
                Event::EpgUpdated(_, Err(e)) => a.toast(format!("Guide update failed: {e}")),
                Event::EpgUpdated(_, Ok(())) => super::epg::refresh(a),
                Event::Playing { title, kind, .. } => {
                    a.playing_id = a.play_req;
                    a.streams.set_playing(a.playing_id);
                    set_playing(a, &title, kind_name(kind));
                    super::epg::push_player(a);
                }
                Event::PlayerStopped => {
                    a.playing_id = None;
                    a.streams.set_playing(None);
                    set_playing(a, "", "");
                    super::epg::push_player(a);
                }
                Event::PlayFailed(msg) | Event::Error(msg) => a.toast(msg),
                Event::ScanStatus(s) => super::browse::scan::on_status(a, s),
                // consumed by the player and browse areas before they get here
                Event::PlayEmbedded(_)
                | Event::UrlReady(_)
                | Event::InfoLoaded { .. }
                | Event::SeriesLoaded { .. }
                | Event::SearchResults { .. }
                | Event::TestResult(_)
                | Event::AccountChecked(..)
                | Event::Sports(_) => {}
            }
        }
    });
}

fn kind_name(kind: PlayerKind) -> &'static str {
    match kind {
        PlayerKind::Embedded => "the built-in player",
        PlayerKind::Mpv => "mpv",
        PlayerKind::Vlc => "VLC",
        PlayerKind::Custom => "custom player",
    }
}

fn set_playing(a: &super::state::SlintApp, title: &str, kind: &str) {
    if let Some(w) = a.win.upgrade() {
        let core = w.global::<Core>();
        core.set_playing_title(title.into());
        core.set_playing_kind(kind.into());
    }
}
