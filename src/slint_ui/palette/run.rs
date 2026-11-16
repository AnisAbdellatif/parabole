//! Runs a chosen palette entry. Slint callbacks are invoked deferred: we are inside `with_app`.
use super::entries::{Action, Entry};
use crate::model::{MediaKind, StreamRow};
use crate::slint_ui::dialogs::appearance::commit;
use crate::slint_ui::state::{Sel, SlintApp};
use crate::slint_ui::{AppWindow, Core, DialogApi, PlayerApi};
use crate::worker::Command;
use slint::ComponentHandle;
use std::time::Duration;

fn defer(a: &SlintApp, f: impl FnOnce(&AppWindow) + 'static) {
    let weak = a.win.clone();
    slint::Timer::single_shot(Duration::ZERO, move || {
        if let Some(w) = weak.upgrade() {
            f(&w);
        }
    });
}

impl SlintApp {
    pub(super) fn run_entry(&mut self, entry: Entry) {
        match entry {
            Entry::Stream(row) => self.palette_open_stream(row),
            Entry::Category(media, source, key) => {
                self.palette_open_category(media, Sel::Category(source, key))
            }
            Entry::Action(act) => self.palette_action(act),
        }
    }

    fn palette_open_stream(&mut self, row: StreamRow) {
        if row.media == MediaKind::Series {
            crate::slint_ui::browse::open_series(self, &row);
            return;
        }
        self.play_req = Some(row.id);
        self.worker.send(Command::Play {
            profile: self.profile(),
            stream_id: row.id,
            kind: None,
        });
    }

    fn palette_open_category(&mut self, media: MediaKind, sel: Sel) {
        if media != self.state.tab || self.sports.open {
            crate::slint_ui::sports::leave(self);
            let tab = MediaKind::ALL.iter().position(|m| *m == media).unwrap_or(0) as i32;
            self.state.tab = media;
            if let Some(w) = self.win.upgrade() {
                w.global::<Core>().set_tab(tab);
            }
            self.reload_categories();
        }
        crate::slint_ui::callbacks::select(self, sel);
    }

    fn palette_action(&mut self, act: Action) {
        match act {
            Action::RefreshAll => self.worker.send(Command::RefreshAll(self.profile())),
            Action::ThemeMode => commit(self, |s| s.mode = (s.mode + 1) % 3),
            Action::Density => commit(self, |s| s.density = (s.density + 1) % 3),
            Action::ReduceMotion => commit(self, |s| s.reduce_motion = !s.reduce_motion),
            Action::ToggleDock => defer(self, |w| w.global::<PlayerApi>().invoke_toggle_dock()),
            Action::OpenDialog(name) => defer(self, move |w| {
                w.global::<DialogApi>().invoke_open_dialog(name.into())
            }),
            Action::SwitchProfile(i) => defer(self, move |w| {
                w.global::<Core>().invoke_profile_selected(i as i32)
            }),
        }
    }
}
