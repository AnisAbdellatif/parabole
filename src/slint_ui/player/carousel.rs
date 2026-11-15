//! Zap carousel: the rows around a channel, open for 2 s after a zap.
use super::super::PlayerApi;
use super::super::StreamItem;
use super::super::state::{SlintApp, with_app};
use crate::model::{MediaKind, StreamRow};
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use std::ops::Range;
use std::time::Duration;

const OPEN_FOR: Duration = Duration::from_secs(2);
/// Rows shown on each side of the current one.
const SPAN: isize = 2;
/// Rows the docked pane lists on each side of the playing one.
const PANE_SPAN: usize = 25;

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

/// List rows around `center` for the pane: up to `PANE_SPAN` each side, no wrap.
fn pane_window(center: usize, n: usize) -> Range<usize> {
    center.saturating_sub(PANE_SPAN)..(center + PANE_SPAN + 1).min(n)
}

/// Rebuilds the docked pane's channel list around the playing row; empty unless live.
pub fn fill_pane(a: &mut SlintApp) {
    let playing = a
        .player
        .session
        .as_ref()
        .filter(|s| s.is_live())
        .and_then(|s| s.stream_id);
    let n = a.streams.row_count();
    let rows: Vec<StreamRow> = playing
        .and_then(|id| (0..n).find(|&i| a.streams.row(i).is_some_and(|r| r.id == id)))
        .map(|c| {
            pane_window(c, n)
                .filter_map(|i| a.streams.row(i))
                .filter(|r| r.media != MediaKind::Series)
                .collect()
        })
        .unwrap_or_default();
    let items: Vec<StreamItem> = rows
        .iter()
        .map(|r| item(a, r, Some(r.id) == playing))
        .collect();
    let current = rows
        .iter()
        .position(|r| Some(r.id) == playing)
        .map_or(-1, |p| p as i32);
    a.player.pane_ids = rows.iter().map(|r| r.id).collect();
    if let Some(w) = a.win.upgrade() {
        let api = w.global::<PlayerApi>();
        // a fresh model each time, so the pane's `changed items` re-centres the list
        api.set_pane_items(ModelRc::new(VecModel::from(items)));
        api.set_pane_current(current);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pane_window_caps_each_side() {
        assert_eq!(pane_window(100, 500), 75..126);
        assert_eq!(pane_window(3, 500), 0..29);
        assert_eq!(pane_window(98, 100), 73..100);
        assert_eq!(pane_window(0, 1), 0..1);
    }
}
