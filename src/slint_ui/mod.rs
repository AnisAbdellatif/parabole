//! Slint front end. Run with `cargo run --features slint-ui --bin iptv-player-slint`.
mod appearance;
mod browse;
mod callbacks;
mod data;
mod dialogs;
mod events;
pub mod fonts;
mod images;
mod palette;
mod player;
mod push;
mod state;
mod stream_model;
mod wiring;

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
    fonts::install_fallback();
    let win = build(&db_path()?)?;
    win.run().context("run event loop")?;
    Ok(())
}

/// Builds the fully wired window on the current thread without showing it or starting the event loop.
pub fn build(path: &std::path::Path) -> Result<AppWindow> {
    let path = path.to_path_buf();
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
    let images = Images::new(|url, size, dec| {
        let _ = slint::invoke_from_event_loop(move || {
            with_app(|a| {
                match dec {
                    Some(d) => a.images.insert(url.clone(), size, d),
                    None => a.images.fail(url.clone(), size),
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
    core.set_mod_label(push::MOD_LABEL.into());
    state.appearance.apply(&win);
    win.global::<BrowseApi>()
        .set_sidebar_w(state.sidebar_width.clamp(200.0, 460.0));
    let api = win.global::<PlayerApi>();
    api.set_dock_bottom(state.player_dock_bottom);
    api.set_pane_w(state.player_pane_w);
    api.set_pane_h(state.player_pane_h);
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
            play_req: None,
            playing_id: None,
            search_text: String::new(),
            search_applied: String::new(),
            search_timer: Default::default(),
            refreshing: Default::default(),
            toasts,
            toast_born: Vec::new(),
            toast_timer: Default::default(),
            next_toast: 0,
            images,
            player: Default::default(),
            browse: Default::default(),
            dialogs: Default::default(),
            palette: Default::default(),
        });
    });
    callbacks::connect(&win);
    dialogs::connect(&win);
    palette::connect(&win);
    player::connect(&win);
    browse::connect(&win);
    wiring::connect(&win);
    with_app(|a| {
        a.reload_all();
        a.worker.send(Command::AutoRefresh(profile));
    });
    win.window().on_close_requested(|| {
        with_app(|a| {
            if let Some(w) = a.win.upgrade() {
                a.state.sidebar_width = w.global::<BrowseApi>().get_sidebar_w();
                let api = w.global::<PlayerApi>();
                a.state.player_dock_bottom = api.get_dock_bottom();
                a.state.player_pane_w = api.get_pane_w();
                a.state.player_pane_h = api.get_pane_h();
            }
            let _ = a.state.save(&a.state_path);
            a.worker.send(Command::Quit);
        });
        slint::CloseRequestResponse::HideWindow
    });
    Ok(win)
}
