//! Zap carousel: the rows around a channel, open for 2 s after a zap.
use super::super::PlayerApi;
use super::super::StreamItem;
use super::super::state::{SlintApp, with_app};
use crate::model::StreamRow;
use slint::{ComponentHandle, Model};
use std::time::Duration;

const OPEN_FOR: Duration = Duration::from_secs(2);
/// Rows shown on each side of the current one.
const SPAN: isize = 2;

fn initials(name: &str) -> String {
    name.split_whitespace()
        .take(2)
        .filter_map(|w| w.chars().next())
        .flat_map(char::to_uppercase)
        .collect()
}

fn item(a: &SlintApp, r: &StreamRow, current: bool) -> StreamItem {
    let logo = r
        .logo
        .as_deref()
        .and_then(|u| a.images.get(u, crate::slint_ui::images::Size::Logo));
    StreamItem {
        name: r.name.as_str().into(),
        meta: r.num.map(|n| n.to_string()).unwrap_or_default().into(),
        initials: initials(&r.name).into(),
        has_logo: logo.is_some(),
        logo: logo.unwrap_or_default(),
        playing: current,
        ..Default::default()
    }
}

/// Opens the carousel centred on list row `center`.
pub fn show(a: &mut SlintApp, center: usize) {
    let n = a.streams.row_count() as isize;
    if n == 0 {
        return;
    }
    let items: Vec<StreamItem> = (-SPAN..=SPAN)
        .filter(|d| n > 2 * d.abs())
        .filter_map(|d| {
            let i = (center as isize + d).rem_euclid(n) as usize;
            a.streams.row(i).map(|r| item(a, &r, d == 0))
        })
        .collect();
    a.player.zap_items.set_vec(items);
    set_open(a, true);
    a.player
        .zap_timer
        .start(slint::TimerMode::SingleShot, OPEN_FOR, || {
            with_app(close);
        });
}

pub fn show_id(a: &mut SlintApp, stream_id: i64) {
    let at =
        (0..a.streams.row_count()).find(|&i| a.streams.row(i).is_some_and(|r| r.id == stream_id));
    if let Some(i) = at {
        show(a, i);
    }
}

/// Opens it around the channel that is playing (digit entry).
pub fn show_playing(a: &mut SlintApp) {
    if let Some(id) = a.player.session.as_ref().and_then(|s| s.stream_id) {
        show_id(a, id);
    }
}

pub fn close(a: &mut SlintApp) {
    a.player.zap_timer.stop();
    set_open(a, false);
}

fn set_open(a: &SlintApp, open: bool) {
    if let Some(w) = a.win.upgrade() {
        w.global::<PlayerApi>().set_zap_open(open);
    }
}
