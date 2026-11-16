//! TV guide: now-line on live rows, per-channel guide panel, player now-line.
pub mod format;
pub mod guide;
mod guide_pick;

use super::PlayerApi;
use super::state::{SlintApp, with_app};
use crate::model::{MediaKind, NowNext, StreamRow};
use slint::ComponentHandle;
use std::collections::HashMap;
use std::time::Duration;

const TICK: Duration = Duration::from_secs(60);

#[derive(Default)]
pub struct EpgArea {
    timer: slint::Timer,
    pub guide: Option<guide::Open>,
}

/// Now/next for the live rows; DB errors give an empty map (the line is a hint).
pub fn row_epg(a: &SlintApp, rows: &[StreamRow]) -> HashMap<i64, NowNext> {
    let ids: Vec<i64> = rows
        .iter()
        .filter(|r| r.media == MediaKind::Live)
        .map(|r| r.id)
        .collect();
    a.db.epg_now_next(&ids, format::now_secs())
        .unwrap_or_default()
}

pub fn refresh(a: &mut SlintApp) {
    let rows: Vec<StreamRow> = (0..a.streams.len())
        .filter_map(|i| a.streams.row(i))
        .collect();
    let map = row_epg(a, &rows);
    a.streams.refresh_epg(map);
    push_player(a);
    guide::push(a);
}

/// Sets the player's "title · N min left" line for the playing live stream.
pub fn push_player(a: &SlintApp) {
    let Some(win) = a.win.upgrade() else {
        return;
    };
    let now = format::now_secs();
    let line = a
        .playing_id
        .and_then(|id| a.db.epg_now_next(&[id], now).ok()?.remove(&id))
        .map(|n| format!("{} · {}", n.now.title, format::remaining(&n.now, now)))
        .unwrap_or_default();
    win.global::<PlayerApi>().set_epg_now(line.into());
}

pub fn install(a: &mut SlintApp) {
    a.epg.timer.start(slint::TimerMode::Repeated, TICK, || {
        with_app(refresh);
    });
}
