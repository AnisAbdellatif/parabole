//! Slint front end. Run with `cargo run --features slint-ui --bin iptv-player-slint`.
mod callbacks;
mod data;
mod events;
mod images;
mod push;
mod state;
mod stream_model;

use crate::db::Db;
use crate::ui::{db_path, install_mpv_script};
use crate::worker::{self, Command};
use anyhow::{Context, Result};
use images::Images;
use slint::{ComponentHandle, ModelRc};
use state::{APP, Persisted, SlintApp, with_app};
use std::rc::Rc;
use std::sync::Arc;
use stream_model::StreamModel;

slint::include_modules!();

pub fn run() -> Result<()> {
    let path = db_path()?;
    install_mpv_script(&path);
    let db = Db::open(&path).context("open database")?;
    let default_profile = db.ensure_default_profile()?;
    let win = AppWindow::new().context("create window")?;
    let worker = worker::spawn(
        path.clone(),
        Arc::new(|| {
            let _ = slint::invoke_from_event_loop(events::drain);
        }),
    )?;
    let images = Images::new(|url, buf| {
        let _ = slint::invoke_from_event_loop(move || {
            with_app(|a| {
                match buf {
                    Some(b) => a.images.insert(url.clone(), b),
                    None => a.images.fail(url.clone()),
                }
                a.streams.image_ready(&url);
            });
        });
    });
    let state_path = path.with_file_name("slint-ui.json");
    let mut state = Persisted::load(&state_path);
    state.profile.get_or_insert(default_profile);
    let streams = Rc::new(StreamModel::new(images.clone()));
    let toasts = Rc::new(slint::VecModel::default());
    let core = win.global::<Core>();
    core.set_streams(ModelRc::from(streams.clone()));
    core.set_toasts(ModelRc::from(toasts.clone()));
    core.set_tab(state.tab as i32);
    let profile = state.profile.unwrap_or_default();
    APP.with(|a| {
        *a.borrow_mut() = Some(SlintApp {
            db,
            worker,
            win: win.as_weak(),
            state,
            state_path,
            profiles: Vec::new(),
            sources: Vec::new(),
            categories: Vec::new(),
            groups: Vec::new(),
            streams,
            search_text: String::new(),
            search_applied: String::new(),
            search_timer: Default::default(),
            refreshing: Default::default(),
            toasts,
            toast_born: Vec::new(),
            toast_timer: Default::default(),
            next_toast: 0,
            images,
        });
    });
    callbacks::connect(&win);
    with_app(|a| {
        a.reload_all();
        a.worker.send(Command::AutoRefresh(profile));
    });
    win.window().on_close_requested(|| {
        with_app(|a| {
            let _ = a.state.save(&a.state_path);
            a.worker.send(Command::Quit);
        });
        slint::CloseRequestResponse::HideWindow
    });
    win.run().context("run event loop")?;
    Ok(())
}
