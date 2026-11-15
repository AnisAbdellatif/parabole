//! `PlayerApi.key`, ported from `ui/player/keys.rs` and `zap.rs`.
use super::super::state::{SlintApp, with_app};
use super::super::{Core, PlayerApi};
use super::{carousel, controls, playback};
use crate::model::{MediaKind, StreamRow};
use crate::player::PlayerKind;
use crate::worker::Command;
use slint::platform::Key;
use slint::{ComponentHandle, Model};
use std::time::Instant;

const SEEK_S: f64 = 10.0;
const VOL_STEP: f64 = 5.0;

/// The parts of a Slint `KeyEvent` this module reads.
struct Pressed {
    text: String,
    /// Control or Cmd.
    cmd: bool,
}

pub fn connect(win: &super::super::AppWindow) {
    win.global::<PlayerApi>().on_key(|e| {
        let e = Pressed {
            text: e.text.to_string(),
            cmd: e.modifiers.control || e.modifiers.meta,
        };
        with_app(|a| handle(a, &e)).unwrap_or(false)
    });
}

fn is(e: &Pressed, k: Key) -> bool {
    e.text == char::from(k).to_string()
}

fn play_embedded(a: &mut SlintApp, stream_id: i64) {
    a.worker.send(Command::Play {
        profile: a.profile(),
        stream_id,
        kind: Some(PlayerKind::Embedded),
    });
}

/// Next row `step` away from the playing one in the current list (wraps, skips series rows).
fn neighbor_row(a: &SlintApp, step: isize) -> Option<(usize, StreamRow)> {
    let playing = a.player.session.as_ref()?.stream_id?;
    let n = a.streams.row_count() as isize;
    let at =
        (0..n as usize).position(|i| a.streams.row(i).is_some_and(|r| r.id == playing))? as isize;
    (1..=n)
        .map(|d| (at + step * d).rem_euclid(n) as usize)
        .filter_map(|i| a.streams.row(i).map(|r| (i, r)))
        .find(|(_, r)| r.media != MediaKind::Series)
}

fn select_row(a: &SlintApp, i: usize) {
    if let Some(w) = a.win.upgrade() {
        w.global::<Core>().set_cursor(i as i32);
    }
}

fn zap(a: &mut SlintApp, step: isize) {
    if let Some((i, row)) = neighbor_row(a, step) {
        select_row(a, i);
        carousel::show(a, i);
        play_embedded(a, row.id);
    }
}

/// Live row for channel number `n`: by its number, or by 1-based position when no row has one.
fn row_for_number(a: &SlintApp, n: i64) -> Option<(usize, StreamRow)> {
    let count = a.streams.row_count();
    let numbered = (0..count).any(|i| a.streams.row(i).is_some_and(|r| r.num.is_some()));
    let at = if numbered {
        (0..count).find(|&i| a.streams.row(i).is_some_and(|r| r.num == Some(n)))?
    } else {
        usize::try_from(n.checked_sub(1)?).ok()?
    };
    a.streams
        .row(at)
        .filter(|r| r.media == MediaKind::Live)
        .map(|r| (at, r))
}

pub fn zap_to_number(a: &mut SlintApp, n: i64) {
    match row_for_number(a, n) {
        Some((i, row)) => {
            select_row(a, i);
            carousel::show(a, i);
            play_embedded(a, row.id);
        }
        None => a.toast(format!("No live channel {n}")),
    }
}

fn seek_by(a: &SlintApp, delta: f64) {
    if let Some(c) = a.player.core.as_ref()
        && let (Some(pos), Some(dur)) = (c.state().time_pos, c.state().duration)
    {
        c.seek_to((pos + delta).clamp(0.0, dur));
    }
}

fn volume_by(a: &mut SlintApp, delta: f64) {
    if let Some(c) = a.player.core.as_mut() {
        let v = (c.state().volume + delta).clamp(0.0, 130.0);
        c.set_volume(v);
    }
}

fn handle(a: &mut SlintApp, e: &Pressed) -> bool {
    let Some(w) = a.win.upgrade() else {
        return false;
    };
    if a.player.session.is_none() {
        return false;
    }
    let api = w.global::<PlayerApi>();
    let focused = a.player.fullscreen || a.player.mini || api.get_hover();
    let handled = handle_keys(a, e, focused, e.cmd);
    if handled {
        super::sync::push(a);
    }
    handled
}

fn handle_keys(a: &mut SlintApp, e: &Pressed, focused: bool, cmd: bool) -> bool {
    let text = e.text.as_str();
    if cmd && is(e, Key::UpArrow) || focused && is(e, Key::PageUp) {
        zap(a, -1);
        return true;
    }
    if cmd && is(e, Key::DownArrow) || focused && is(e, Key::PageDown) {
        zap(a, 1);
        return true;
    }
    if cmd && text.eq_ignore_ascii_case("l") {
        playback::go_live(a);
        return true;
    }
    if cmd {
        return false;
    }
    if focused && text.len() == 1 && text.as_bytes()[0].is_ascii_digit() {
        a.player
            .zap
            .push(text.chars().next().unwrap_or('0'), Instant::now());
        carousel::show_playing(a);
        return true;
    }
    if a.player.zap.text().is_some() {
        if is(e, Key::Escape) {
            a.player.zap.cancel();
            return true;
        }
        if is(e, Key::Return) {
            if let Some(n) = a.player.zap.take() {
                zap_to_number(a, n);
            }
            return true;
        }
    }
    if is(e, Key::Backspace) || text.eq_ignore_ascii_case("l") {
        if let Some(prev) = a.player.prev_stream {
            carousel::show_id(a, prev);
            play_embedded(a, prev);
            return true;
        }
        return false;
    }
    if is(e, Key::Escape) && (a.player.fullscreen || a.player.mini) {
        if a.player.fullscreen {
            controls::set_fullscreen(a, false);
        } else {
            controls::set_mini(a, false);
        }
        return true;
    }
    if !focused {
        return false;
    }
    match text {
        " " => {
            if let Some(c) = a.player.core.as_mut() {
                let paused = c.state().paused;
                c.set_paused(!paused);
            }
        }
        "m" | "M" => controls::toggle_mute(a),
        "p" | "P" => controls::toggle_mini(a),
        "i" | "I" => controls::toggle_stats(a),
        "f" | "F" if !a.player.mini => controls::toggle_fullscreen(a),
        _ if is(e, Key::LeftArrow) => seek_by(a, -SEEK_S),
        _ if is(e, Key::RightArrow) => seek_by(a, SEEK_S),
        _ if is(e, Key::UpArrow) => volume_by(a, VOL_STEP),
        _ if is(e, Key::DownArrow) => volume_by(a, -VOL_STEP),
        _ => return false,
    }
    true
}
