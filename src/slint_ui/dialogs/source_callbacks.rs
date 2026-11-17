//! Sources dialog callbacks.
use super::source_form::detect_paste;
use super::ua_preset::PRESETS;
use super::{AppWindow, DialogApi};
use crate::slint_ui::state::with_app;
use crate::worker::{Command, auto_refresh};
use slint::ComponentHandle;

use super::sources::{CUSTOM_UA, reset_form};

pub fn connect(win: &AppWindow) {
    let api = win.global::<DialogApi>();
    api.on_sources_refresh(|id| {
        with_app(|a| a.worker.send(Command::Refresh(id.into())));
    });
    api.on_sources_refresh_all(|| {
        with_app(|a| a.worker.send(Command::RefreshAll(a.profile())));
    });
    api.on_sources_check(|id| {
        with_app(|a| {
            a.dialogs.checking.insert(id.into());
            a.worker.send(Command::CheckAccount(id.into()));
            a.push_sources();
        });
    });
    api.on_sources_edit(|id| {
        with_app(|a| a.load_source_into_form(id.into()));
    });
    api.on_sources_delete(|id| {
        with_app(|a| {
            if a.dialogs.confirm_source == Some(id.into()) {
                a.dialogs.confirm_source = None;
                a.worker.send(Command::DeleteSource(id.into()));
            } else {
                a.dialogs.confirm_source = Some(id.into());
            }
            a.push_sources();
        });
    });
    api.on_sources_cancel_delete(|| {
        with_app(|a| {
            a.dialogs.confirm_source = None;
            a.push_sources();
        });
    });
    api.on_form_paste_edited(|paste| {
        with_app(|a| {
            let Some(w) = a.win.upgrade() else { return };
            let api = w.global::<DialogApi>();
            let (msg, xtream, fill) = detect_paste(&paste);
            api.set_form_detected(msg.into());
            api.set_form_is_xtream(xtream);
            if let Some((url, user, pass)) = fill {
                api.set_form_url(url.into());
                api.set_form_user(user.into());
                api.set_form_pass(pass.into());
            }
        });
    });
    let weak = win.as_weak();
    api.on_form_choose_file(move || {
        let picked = rfd::FileDialog::new()
            .add_filter("Playlist", &["m3u", "m3u8"])
            .pick_file();
        if let (Some(p), Some(w)) = (picked, weak.upgrade()) {
            w.global::<DialogApi>()
                .set_form_url(p.to_string_lossy().into_owned().into());
        }
    });
    let weak = win.as_weak();
    api.on_form_ua_selected(move |i| {
        let Some(w) = weak.upgrade() else { return };
        match i {
            1..=4 => w
                .global::<DialogApi>()
                .set_form_ua(PRESETS[i as usize - 1].1.into()),
            CUSTOM_UA => {}
            _ => w.global::<DialogApi>().set_form_ua("".into()),
        }
    });
    api.on_form_submit(|| {
        with_app(|a| a.submit_source_form());
    });
    api.on_form_cancel(|| {
        with_app(|a| {
            a.dialogs.editing = None;
            if let Some(w) = a.win.upgrade() {
                reset_form(&w.global::<DialogApi>());
            }
        });
    });
    api.on_form_test(|| {
        with_app(|a| a.test_source_form());
    });
    api.on_auto_hours_commit(|text| {
        with_app(|a| {
            let hours = text.trim().parse::<u32>().unwrap_or(0).min(720);
            a.write(move |db| auto_refresh::set_hours(db, hours), |_| {});
            // committed on every edit: leave an empty field alone while the user retypes it
            let shown = hours.to_string();
            if let Some(w) = a.win.upgrade()
                && !text.trim().is_empty()
                && text.trim() != shown
            {
                w.global::<DialogApi>().set_auto_hours(shown.into());
            }
        });
    });
}
