//! Session lifecycle and the housekeeping tick; mirrors `ui/player/mod.rs` (`embed_tick`).
use super::super::Core;
use super::super::state::{SlintApp, with_app};
use super::sync;
use crate::player::{Notice, PlayerKind};
use crate::ui::player::autonext;
use crate::ui::player::session::{EndAction, SAVE_EVERY, STABLE_AFTER_S, Session, end_action};
use crate::ui::player::track_memory;
use crate::worker::{Command, EmbedPlay, Track, player_settings};
use slint::ComponentHandle;
use std::time::{Duration, Instant};

const TICK: Duration = Duration::from_millis(250);

pub fn on_embed_play(a: &mut SlintApp, play: EmbedPlay) {
    save_position(a);
    a.player.settings = player_settings::load(&a.db);
    let ar = &mut a.player;
    let result = match ar.core.as_mut().filter(|_| ar.video_ready) {
        Some(core) => core.play(&ar.settings, &play.req),
        None => Err(anyhow::anyhow!(
            ar.init_error
                .clone()
                .unwrap_or_else(|| "still starting".into())
        )),
    };
    match result {
        Ok(()) => {
            let old = ar.session.as_ref().and_then(|s| s.stream_id);
            if old.is_some() && old != play.stream_id {
                ar.prev_stream = old;
            }
            set_playing(a, &play.req.title, "the built-in player");
            a.player.session = Some(Session::new(play));
            a.player.timer.start(slint::TimerMode::Repeated, TICK, || {
                with_app(tick);
            });
            sync::push(a);
        }
        Err(e) => {
            a.toast(format!("Embedded player unavailable ({e:#}); using mpv"));
            a.worker.send(Command::LaunchRequest {
                req: play.req,
                kind: PlayerKind::Mpv,
                track: play.track,
                stream_id: play.stream_id,
            });
        }
    }
}

pub fn stop_embedded(a: &mut SlintApp) {
    save_position(a);
    if let Some(core) = a.player.core.as_mut() {
        core.stop();
    }
    a.player.session = None;
    a.player.timer.stop();
    a.player.zap.cancel();
    super::controls::leave_special_modes(a);
    set_playing(a, "", "");
    sync::push(a);
}

fn set_playing(a: &SlintApp, title: &str, kind: &str) {
    if let Some(w) = a.win.upgrade() {
        let core = w.global::<Core>();
        core.set_playing_title(title.into());
        core.set_playing_kind(kind.into());
    }
}

pub fn save_position(a: &mut SlintApp) {
    let Some(s) = a.player.session.as_mut().filter(|s| !s.is_live()) else {
        return;
    };
    if let (Some(track), Some((pos, dur))) = (s.track.clone(), s.known) {
        a.worker.send(Command::SavePosition {
            track,
            position_s: pos,
            duration_s: dur,
        });
        s.last_save = Instant::now();
    }
}

/// Mpv events, live retry, periodic resume saves, auto-next and zap timers.
pub fn tick(a: &mut SlintApp) {
    zap_idle(a);
    autonext_due(a);
    let SlintApp {
        player, db, worker, ..
    } = a;
    let (Some(core), Some(s)) = (player.core.as_mut(), player.session.as_mut()) else {
        if let Some(c) = player.core.as_mut() {
            c.poll();
        }
        return;
    };
    let mut finished = false;
    let mut give_up = None;
    for notice in core.poll() {
        let Notice::Ended { failed } = notice else {
            continue;
        };
        match end_action(s.req.media, s.attempt, failed) {
            EndAction::Retry(d) => {
                s.attempt += 1;
                s.retry_at = Some(Instant::now() + d);
            }
            EndAction::GiveUp => {
                s.error = Some("The stream stopped and could not be reopened".into());
                give_up = s.stream_id.filter(|_| s.is_live());
            }
            EndAction::Finish => finished = true,
        }
    }
    if let Some(pos) = core.state().time_pos {
        if pos > STABLE_AFTER_S {
            s.attempt = 0;
            s.error = None;
        }
        if let Some(dur) = core.state().duration {
            s.known = Some((pos, dur));
        }
    }
    if s.retry_at.is_some_and(|at| Instant::now() >= at) {
        s.retry_at = None;
        if let Err(e) = core.play(&player.settings, &s.req) {
            s.error = Some(format!("{e:#}"));
        }
    }
    if finished {
        if let (Some(track), Some((_, dur))) = (s.track.clone(), s.known) {
            worker.send(Command::SavePosition {
                track,
                position_s: dur,
                duration_s: dur,
            });
            s.known = Some((dur, dur));
        }
        s.autonext = autonext::find_next(s);
        if s.autonext.is_none() {
            stop_embedded(a);
        } else {
            sync::push(a);
        }
        return;
    }
    track_memory::tick(db, core, s);
    let due_save = !s.is_live() && s.last_save.elapsed() >= SAVE_EVERY;
    if due_save {
        save_position(a);
    }
    if let Some(id) = give_up {
        switch_to_alternate(a, id);
    }
    sync::push(a);
}

fn zap_idle(a: &mut SlintApp) {
    if let Some(n) = a.player.zap.take_if_idle(Instant::now()) {
        super::keys::zap_to_number(a, n);
    }
}

fn autonext_due(a: &mut SlintApp) {
    let due = a
        .player
        .session
        .as_ref()
        .and_then(|s| s.autonext.as_ref())
        .is_some_and(|n| Instant::now() >= n.deadline);
    if due {
        autonext_choice(a, true);
    }
}

/// `play_now` false cancels (the player stops).
pub fn autonext_choice(a: &mut SlintApp, play_now: bool) {
    let Some(s) = a.player.session.as_mut() else {
        return;
    };
    let Some(next) = s.autonext.take() else {
        return;
    };
    let cmd = play_now
        .then(|| {
            s.series.as_ref().zip(s.track.as_ref()).and_then(
                |(
                    series,
                    Track {
                        profile, source, ..
                    },
                )| { series.play_command(*profile, *source, &next.episode_id) },
            )
        })
        .flatten();
    match cmd {
        Some(cmd) => a.worker.send(cmd),
        None => stop_embedded(a),
    }
}

/// When a live stream gives up for good, try the same channel from another source once
/// (see `ui/player/alternate.rs`).
fn switch_to_alternate(a: &mut SlintApp, failed: i64) {
    if a.player.switched_to == Some(failed) {
        a.player.switched_to = None;
        return;
    }
    let profile = a.profile();
    let alt = match a.db.alternates(profile, failed) {
        Ok(alts) => alts.into_iter().next(),
        Err(e) => return a.toast(format!("{e:#}")),
    };
    let Some(alt) = alt else {
        return;
    };
    a.player.switched_to = Some(alt.stream_id);
    a.toast(format!("Switched to {}", alt.source_name));
    a.worker.send(Command::Play {
        profile,
        stream_id: alt.stream_id,
        kind: None,
    });
}

/// Go live: reload a live stream so it lands on the live edge.
pub fn go_live(a: &mut SlintApp) {
    let ar = &mut a.player;
    if let (Some(core), Some(s)) = (ar.core.as_mut(), ar.session.as_mut())
        && s.is_live()
        && core.reload(&ar.settings).is_ok()
    {
        s.attempt = 0;
        s.error = None;
    }
}
