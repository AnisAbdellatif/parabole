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

pub(crate) fn clear_search(a: &mut SlintApp) {
    a.search_text.clear();
    a.search_applied.clear();
    a.browse.filter = Default::default();
    a.search_timer.stop();
    if let Some(w) = a.win.upgrade() {
        w.global::<Core>().set_search_text("".into());
    }
}

pub(crate) fn select(a: &mut SlintApp, sel: Sel) {
    a.state.sel = Some(sel);
    clear_search(a);
    reset_cursor(a);
    super::browse::close_series(a);
    a.push_sidebar();
    a.reload_streams();
}

pub(crate) fn switch_tab(a: &mut SlintApp, i: i32) {
    if i == 3 {
        if a.sports.settings.enabled {
            a.state.smart = false;
        }
        super::sports::open(a);
        return;
    }
    super::sports::leave(a);
    a.state.smart = i == 4;
    a.state.tab = match i {
        1 => MediaKind::Movie,
        2 => MediaKind::Series,
        _ => MediaKind::Live,
    };
    a.state.sel = None;
    reset_cursor(a);
    super::browse::close_series(a);
    if let Some(w) = a.win.upgrade() {
        w.global::<Core>().set_tab(i);
        w.global::<super::BrowseApi>().set_sidebar_focus(false);
    }
    a.reload_categories();
}

pub fn connect(win: &AppWindow) {
    let core = win.global::<Core>();
    core.on_tab_selected(|i| {
        with_app(|a| switch_tab(a, i));
    });
    core.on_profile_selected(|i| {
        with_app(|a| {
            a.state.profile = a.profiles.get(i as usize).map(|p| p.id);
            a.state.source = None;
            a.state.sel = None;
            clear_search(a);
            super::browse::close_series(a);
            a.push_profiles();
            a.reload_sources();
        });
    });
    core.on_source_selected(|i| {
        with_app(|a| {
            let at = (i as usize).checked_sub(1);
            a.state.source = at.and_then(|n| a.sources.get(n)).map(|s| s.id);
            clear_search(a);
            reset_cursor(a);
            super::browse::close_series(a);
            a.reload_categories();
        });
    });
    core.on_search_edited(|s| {
        with_app(|a| {
            a.search_text = s.to_string();
            a.search_timer
                .start(slint::TimerMode::SingleShot, SEARCH_DELAY, || {
                    with_app(|a| {
                        a.search_applied = a.search_text.trim().to_owned();
                        super::browse::close_series(a);
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
            if super::browse::scan::action(a, &id) {
                return;
            }
            if let Some(sel) = Sel::from_id(&id) {
                select(a, sel);
            }
        });
    });
    core.on_stream_activated(|i| {
        with_app(|a| {
            super::browse::activate(a, i as usize);
        });
    });
    core.on_favorite_toggled(|i| {
        with_app(|a| {
            super::browse::row_action(a, i as usize, super::browse::RowAction::ToggleFavorite);
        });
    });
    core.on_toast_dismissed(|id| {
        with_app(|a| a.remove_toast(id));
    });
    core.on_stop_playback(|| {
        with_app(|a| a.worker.send(Command::StopPlayer));
    });
}
