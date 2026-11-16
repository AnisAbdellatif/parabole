//! Player area: embedded libmpv drawn into the Slint window. Spec: docs/slint-phase2.md.
mod autonext;
mod carousel;
mod chrome;
mod controls;
mod keys;
mod pip;
mod playback;
mod quality;
mod scrobble;
mod session;
mod sync;
mod track_memory;
mod video;
mod zap;

use super::state::{SlintApp, with_app};
use super::{AppWindow, StreamItem, TrackItem};
use crate::player::embed::MpvCore;
use crate::player::{PlayerSettings, TrackInfo};
use crate::worker::{Command, Event, player_settings};
use session::Session;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use zap::ZapBuffer;

/// Per-area UI-thread state, stored in `SlintApp`.
#[derive(Default)]
pub struct PlayerArea {
    core: Option<MpvCore>,
    /// Why the embedded player can't be used (libmpv or GL failed); None while it works.
    init_error: Option<String>,
    /// The GL side is up: a frame can be rendered.
    video_ready: bool,
    settings: PlayerSettings,
    session: Option<Session>,
    zap: ZapBuffer,
    prev_stream: Option<i64>,
    /// Stream picked by the automatic source switch; it never switches again.
    switched_to: Option<i64>,
    fullscreen: bool,
    mini: bool,
    /// Window geometry to restore when leaving picture-in-picture.
    windowed: Option<pip::Geometry>,
    zap_items: Rc<slint::VecModel<StreamItem>>,
    zap_timer: slint::Timer,
    /// Stream ids behind `PlayerApi.pane-items`, same order.
    pane_ids: Vec<i64>,
    timer: slint::Timer,
    audio: Rc<slint::VecModel<TrackItem>>,
    subs: Rc<slint::VecModel<TrackItem>>,
    shown_tracks: Vec<TrackInfo>,
}

impl Drop for PlayerArea {
    fn drop(&mut self) {
        // Cmd+Q exits before GL teardown frees the render context; mpv_destroy would abort. The OS reclaims it.
        std::mem::forget(self.core.take());
    }
}

/// Registers this area's `PlayerApi` callbacks and the GL notifier. Called once from `run()`.
pub fn connect(win: &AppWindow) {
    controls::connect(win);
    keys::connect(win);
    pip::connect(win);
    let pending = Arc::new(AtomicBool::new(false));
    let wake = Arc::clone(&pending);
    // Coalesces mpv wakeups: at most one poll is queued on the UI thread.
    let settings = with_app(|a| player_settings::load(&a.db)).unwrap_or_default();
    let core = MpvCore::new(&settings, move || {
        if !wake.swap(true, Ordering::AcqRel) {
            let pending = Arc::clone(&wake);
            let _ = slint::invoke_from_event_loop(move || {
                pending.store(false, Ordering::Release);
                with_app(playback::tick);
            });
        }
    });
    with_app(|a| sync::init_models(a, win));
    let core = match core {
        Ok(c) => c,
        Err(e) => return fail(format!("{e:#}")),
    };
    let mpv: *const libmpv2::Mpv = core.mpv();
    with_app(|a| a.player.core = Some(core));
    if let Err(e) = video::install(win, mpv, ready_later, fail_later) {
        fail(format!("{e}"));
    }
}

fn fail(reason: String) {
    with_app(|a| a.player.init_error = Some(reason));
}

fn ready_later() {
    let _ = slint::invoke_from_event_loop(|| {
        with_app(|a| a.player.video_ready = true);
    });
}

/// For use inside the rendering notifier: defers so it never runs inside a render pass.
fn fail_later(reason: String) {
    let _ = slint::invoke_from_event_loop(move || fail(reason));
}

/// Stops whatever is playing: the embedded session and any external player.
pub fn stop_all(a: &mut SlintApp) {
    if a.player.session.is_some() {
        playback::stop_embedded(a);
    }
    a.worker.send(Command::StopPlayer);
}

/// Sees every worker event first; return it to let the next area and core handle it too.
pub fn on_event(a: &mut SlintApp, ev: Event) -> Option<Event> {
    match ev {
        Event::PlayEmbedded(play) => playback::on_embed_play(a, *play),
        // an external player took over
        Event::Playing { .. } if a.player.session.is_some() => {
            playback::stop_embedded(a);
            return Some(ev);
        }
        // the external player stopping must not clear the embedded session's status
        Event::PlayerStopped if a.player.session.is_some() => {}
        other => return Some(other),
    }
    None
}
