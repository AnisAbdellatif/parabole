//! Dialog area. Ownership and spec: docs/slint-phase2.md.
pub(crate) mod appearance;
mod group;
mod guides;
mod profiles;
mod scan;
mod settings;
mod source_callbacks;
mod source_form;
mod sources;
mod trakt;
mod ua_preset;

use super::state::{SlintApp, with_app};
use super::{AppWindow, DialogApi};
use crate::model::{ProfileId, SourceId};
use crate::worker::Event;
use slint::ComponentHandle;
use std::collections::HashSet;
use std::time::Duration;

/// Per-area UI-thread state, stored in `SlintApp`.
#[derive(Default)]
pub struct DialogArea {
    editing: Option<SourceId>,
    checking: HashSet<SourceId>,
    confirm_source: Option<SourceId>,
    renaming: Option<ProfileId>,
    confirm_profile: Option<ProfileId>,
    scan: Vec<scan::Entry>,
}

/// Registers this area's `DialogApi` callbacks. Called once from `run()`.
pub fn connect(win: &AppWindow) {
    win.global::<DialogApi>().on_open_dialog(|name| {
        with_app(|a| a.open_dialog(&name));
    });
    source_callbacks::connect(win);
    settings::connect(win);
    guides::connect(win);
    profiles::connect(win);
    group::connect(win);
    appearance::connect(win);
    scan::connect(win);
    trakt::connect(win);
}

impl SlintApp {
    fn open_dialog(&mut self, name: &str) {
        if !matches!(
            name,
            "sources" | "settings" | "profiles" | "appearance" | "group"
        ) {
            return;
        }
        let (open, tab) = match name {
            "appearance" => ("settings", 2),
            n => (n, 0),
        };
        // open first: push_sources skips unless DialogApi.open is "sources"
        if let Some(w) = self.win.upgrade() {
            w.global::<DialogApi>().set_open(open.into());
        }
        match open {
            "sources" => self.load_sources_dialog(),
            "settings" => self.load_settings_dialog(tab),
            "profiles" => self.load_profiles_dialog(),
            _ => {}
        }
    }

    fn close_dialog(&self) {
        if let Some(w) = self.win.upgrade() {
            w.global::<DialogApi>().set_open("".into());
        }
    }
}

/// Sees every worker event first; return it to let the next area and core handle it too.
pub fn on_event(a: &mut SlintApp, ev: Event) -> Option<Event> {
    match ev {
        Event::AccountChecked(id, res) => {
            a.dialogs.checking.remove(&id);
            if let Err(e) = res {
                a.toast(format!("Account check failed: {e}"));
            }
            a.push_sources();
            None
        }
        Event::Trakt(update) => {
            trakt::on_event(a, &update);
            None
        }
        Event::TestResult(res) => {
            a.set_test_result(res);
            None
        }
        Event::EpgUpdated(..) | Event::EpgRelinked => {
            if a.win
                .upgrade()
                .is_some_and(|w| w.global::<DialogApi>().get_open() == "settings")
            {
                a.push_guides();
            }
            Some(ev)
        }
        Event::SourceChanged | Event::RefreshStarted(_) | Event::RefreshDone(..) => {
            // Core updates `sources`/`refreshing` after this returns, so refresh the list afterwards.
            slint::Timer::single_shot(Duration::ZERO, || {
                with_app(|a| a.push_sources());
            });
            Some(ev)
        }
        ev => Some(ev),
    }
}
