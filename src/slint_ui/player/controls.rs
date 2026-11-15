//! `PlayerApi` control callbacks and fullscreen/mini window modes.
use super::super::state::{SlintApp, with_app};
use super::super::{AppWindow, PlayerApi};
use super::{pip, playback, sync};
use crate::player::TrackKind;
use slint::ComponentHandle;
use std::time::Duration;

const FULLSCREEN_EXIT: Duration = Duration::from_millis(900);
/// Volume slider range, as in the egui bar.
const VOL_MAX: f64 = 130.0;

pub fn connect(win: &AppWindow) {
    let api = win.global::<PlayerApi>();
    api.on_toggle_pause(|| {
        with_app(|a| {
            if let Some(c) = a.player.core.as_mut() {
                let paused = c.state().paused;
                c.set_paused(!paused);
            }
            sync::push(a);
        });
    });
    api.on_seek(|frac| {
        with_app(|a| {
            if let Some(c) = a.player.core.as_ref()
                && let Some(d) = c.state().duration
            {
                c.seek_to(f64::from(frac.clamp(0.0, 1.0)) * d);
            }
        });
    });
    api.on_set_volume(|frac| {
        with_app(|a| {
            if let Some(c) = a.player.core.as_mut() {
                c.set_volume(f64::from(frac.clamp(0.0, 1.0)) * VOL_MAX);
            }
            sync::push(a);
        });
    });
    api.on_toggle_mute(|| with_app(toggle_mute).unwrap_or_default());
    api.on_select_audio(|id| select(TrackKind::Audio, id));
    api.on_select_sub(|id| select(TrackKind::Sub, id));
    api.on_stop(|| with_app(playback::stop_embedded).unwrap_or_default());
    api.on_toggle_fullscreen(|| with_app(toggle_fullscreen).unwrap_or_default());
    api.on_toggle_mini(|| with_app(toggle_mini).unwrap_or_default());
    api.on_toggle_stats(|| with_app(toggle_stats).unwrap_or_default());
    api.on_toggle_dock(|| {
        if let Some(w) = with_app(|a| a.win.upgrade()).flatten() {
            let api = w.global::<PlayerApi>();
            api.set_dock_bottom(!api.get_dock_bottom());
        }
    });
    api.on_go_live(|| {
        with_app(playback::go_live);
    });
    api.on_next_cancel(|| {
        with_app(|a| playback::autonext_choice(a, false));
    });
    api.on_next_now(|| {
        with_app(|a| playback::autonext_choice(a, true));
    });
}

pub fn toggle_mute(a: &mut SlintApp) {
    if let Some(c) = a.player.core.as_mut() {
        let muted = c.state().muted;
        c.set_muted(!muted);
    }
    sync::push(a);
}

pub fn toggle_stats(a: &mut SlintApp) {
    if let Some(w) = a.win.upgrade() {
        let api = w.global::<PlayerApi>();
        api.set_show_stats(!api.get_show_stats());
    }
    sync::push(a);
}

fn select(kind: TrackKind, id: i32) {
    with_app(|a| {
        if let Some(c) = a.player.core.as_ref() {
            c.select_track(kind, (id >= 0).then_some(i64::from(id)));
        }
    });
}

pub fn toggle_fullscreen(a: &mut SlintApp) {
    set_fullscreen(a, !a.player.fullscreen);
}

pub fn toggle_mini(a: &mut SlintApp) {
    set_mini(a, !a.player.mini);
}

pub fn set_fullscreen(a: &mut SlintApp, on: bool) {
    if on && a.player.mini {
        // PiP restores the geometry saved before it; keep it for the next PiP
        let keep = a.player.windowed.clone();
        set_mini(a, false);
        a.player.windowed = keep;
    } else if on && let Some(w) = a.win.upgrade() {
        a.player.windowed = Some(pip::save(&w));
    }
    a.player.fullscreen = on;
    if let Some(w) = a.win.upgrade() {
        // the window system may re-enter callbacks, so apply after this borrow ends
        let weak = w.as_weak();
        slint::Timer::single_shot(Duration::ZERO, move || {
            if let Some(w) = weak.upgrade() {
                w.window().set_fullscreen(on);
            }
        });
    }
    sync::push(a);
}

pub fn set_mini(a: &mut SlintApp, on: bool) {
    if on == a.player.mini {
        return;
    }
    let from_fullscreen = on && a.player.fullscreen;
    if from_fullscreen {
        set_fullscreen(a, false);
    }
    a.player.mini = on;
    if let Some(w) = a.win.upgrade() {
        if on && !from_fullscreen {
            a.player.windowed = Some(pip::save(&w));
        }
        let restore = if on { None } else { a.player.windowed.take() };
        // shortcut: fixed wait for macOS' fullscreen-exit animation; poll the window size to do it right
        let delay = if from_fullscreen {
            FULLSCREEN_EXIT
        } else {
            Duration::ZERO
        };
        let weak = w.as_weak();
        slint::Timer::single_shot(delay, move || {
            let Some(w) = weak.upgrade() else {
                return;
            };
            match (on, restore) {
                (true, _) => pip::enter(&w),
                (false, Some(geometry)) => pip::restore(&w, &geometry),
                (false, None) => {}
            }
        });
    }
    sync::push(a);
}

/// Called when the session ends: back to the normal window.
pub fn leave_special_modes(a: &mut SlintApp) {
    if a.player.fullscreen {
        set_fullscreen(a, false);
    }
    set_mini(a, false);
}
