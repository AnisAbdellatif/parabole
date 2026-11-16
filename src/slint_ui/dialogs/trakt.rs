//! Trakt row of the Services tab: sign-in by code, disconnect, "report what I watch".
use super::{AppWindow, DialogApi};
use crate::slint_ui::state::{SlintApp, with_app};
use crate::worker::trakt::store;
use crate::worker::{Command, TraktUpdate};
use slint::ComponentHandle;
use std::process::Command as Process;

const NOT_CONNECTED: i32 = 0;
const WAITING: i32 = 1;
const CONNECTED: i32 = 2;

pub fn connect(win: &AppWindow) {
    let api = win.global::<DialogApi>();
    api.on_trakt_connect(|| {
        with_app(|a| {
            let p = a.profile();
            a.worker.send(Command::TraktConnect(p));
        });
    });
    api.on_trakt_cancel(|| {
        with_app(|a| a.worker.send(Command::TraktCancel));
    });
    api.on_trakt_disconnect(|| {
        with_app(|a| {
            let p = a.profile();
            a.worker.send(Command::TraktDisconnect(p));
        });
    });
    api.on_trakt_open(|| {
        with_app(|a| {
            if let Some(w) = a.win.upgrade() {
                open_url(&w.global::<DialogApi>().get_trakt_url());
            }
        });
    });
    api.on_trakt_copy(|| {
        with_app(|a| {
            let code = a
                .win
                .upgrade()
                .map(|w| w.global::<DialogApi>().get_trakt_code());
            if let Some(code) = code {
                crate::slint_ui::browse::copy_text(a, &code, "Code copied");
            }
        });
    });
}

/// Opens an https address in the default browser; no shell is involved.
fn open_url(url: &str) {
    if !url.starts_with("https://") {
        return;
    }
    let program = if cfg!(target_os = "macos") {
        "open"
    } else if cfg!(windows) {
        "explorer"
    } else {
        "xdg-open"
    };
    let _ = Process::new(program).arg(url).spawn();
}

/// Shows the stored sign-in of the current profile; keeps a code the user is still entering.
pub fn load(a: &SlintApp) {
    let Some(w) = a.win.upgrade() else { return };
    let api = w.global::<DialogApi>();
    let profile = a.profile();
    api.set_set_trakt_scrobble(store::scrobble_enabled(&a.db, profile));
    match store::tokens(&a.db, profile) {
        Some(t) => {
            api.set_trakt_user(t.username.unwrap_or_default().into());
            api.set_trakt_state(CONNECTED);
        }
        None if api.get_trakt_state() == WAITING => {}
        None => api.set_trakt_state(NOT_CONNECTED),
    }
}

pub fn on_event(a: &mut SlintApp, update: &TraktUpdate) {
    let profile = a.profile();
    let Some(w) = a.win.upgrade() else { return };
    let api = w.global::<DialogApi>();
    match update {
        TraktUpdate::Code {
            profile: p,
            user_code,
            url,
        } if *p == profile => {
            api.set_trakt_code(user_code.as_str().into());
            api.set_trakt_url(url.as_str().into());
            api.set_trakt_error("".into());
            api.set_trakt_state(WAITING);
        }
        TraktUpdate::Connected {
            profile: p,
            username,
        } if *p == profile => {
            api.set_trakt_user(username.as_str().into());
            api.set_trakt_error("".into());
            api.set_trakt_state(CONNECTED);
            a.toast(if username.is_empty() {
                "Trakt connected".to_string()
            } else {
                format!("Trakt connected as @{username}")
            });
        }
        TraktUpdate::Disconnected { profile: p } if *p == profile => {
            api.set_trakt_error("".into());
            api.set_trakt_state(NOT_CONNECTED);
        }
        TraktUpdate::Failed { profile: p, error } if *p == profile => {
            api.set_trakt_error(error.as_str().into());
            api.set_trakt_state(NOT_CONNECTED);
        }
        _ => {}
    }
}
