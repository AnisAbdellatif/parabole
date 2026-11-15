//! Core callbacks -> app logic. Every handler borrows the app through `with_app`.
use super::state::{Sel, SlintApp, with_app};
use super::{AppWindow, Core};
use crate::model::MediaKind;
use crate::worker::Command;
use slint::ComponentHandle;
use std::time::Duration;

const SEARCH_DELAY: Duration = Duration::from_millis(150);

fn reset_cursor(a: &SlintApp) {
    if let Some(w) = a.win.upgrade() {
        w.global::<Core>().set_cursor(0);
    }
}

fn clear_search(a: &mut SlintApp) {
    a.search_text.clear();
    a.search_applied.clear();
    a.search_timer.stop();
    if let Some(w) = a.win.upgrade() {
        w.global::<Core>().set_search_text("".into());
    }
}

pub fn connect(win: &AppWindow) {
    let core = win.global::<Core>();
    core.on_tab_selected(|i| {
        with_app(|a| {
            a.state.tab = match i {
                1 => MediaKind::Movie,
                2 => MediaKind::Series,
                _ => MediaKind::Live,
            };
            a.state.sel = None;
            reset_cursor(a);
            a.reload_categories();
        });
    });
    core.on_profile_selected(|i| {
        with_app(|a| {
            a.state.profile = a.profiles.get(i as usize).map(|p| p.id);
            a.state.source = None;
            a.state.sel = None;
            clear_search(a);
            a.push_profiles();
            a.reload_sources();
        });
    });
    core.on_search_edited(|s| {
        with_app(|a| {
            a.search_text = s.to_string();
            a.search_timer
                .start(slint::TimerMode::SingleShot, SEARCH_DELAY, || {
                    with_app(|a| {
                        a.search_applied = a.search_text.trim().to_owned();
                        reset_cursor(a);
                        a.reload_streams();
                    });
                });
        });
    });
    core.on_search_cleared(|| {
        with_app(|a| {
            clear_search(a);
            a.reload_streams();
        });
    });
    core.on_sidebar_selected(|id| {
        with_app(|a| {
            let Some(sel) = Sel::from_id(&id) else { return };
            a.state.sel = Some(sel);
            clear_search(a);
            reset_cursor(a);
            a.push_sidebar();
            a.reload_streams();
        });
    });
    core.on_stream_activated(|i| {
        with_app(|a| {
            let Some(row) = a.streams.row(i as usize).filter(|r| !r.missing) else {
                return;
            };
            if row.media == MediaKind::Series {
                a.toast("Series view not yet in Slint UI");
                return;
            }
            a.worker.send(Command::Play {
                profile: a.profile(),
                stream_id: row.id,
                kind: None,
            });
        });
    });
    core.on_favorite_toggled(|i| {
        with_app(|a| {
            let Some(row) = a.streams.row(i as usize) else {
                return;
            };
            let res = a.db.set_favorite(
                a.profile(),
                row.source_id,
                row.media,
                &row.key,
                !row.favorite,
            );
            a.report(res);
            a.reload_streams();
        });
    });
    core.on_toast_dismissed(|id| {
        with_app(|a| a.remove_toast(id));
    });
    core.on_stop_playback(|| {
        with_app(|a| a.worker.send(Command::StopPlayer));
    });
}
