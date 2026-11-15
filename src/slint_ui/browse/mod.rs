//! Browse area. Ownership and spec: docs/slint-phase2.md.
use super::AppWindow;
use super::state::SlintApp;
use crate::worker::Event;

/// Per-area UI-thread state, stored in `SlintApp`.
#[derive(Default)]
pub struct BrowseArea {}

/// Registers this area's `BrowseApi` callbacks. Called once from `run()`.
pub fn connect(_win: &AppWindow) {}

/// Sees every worker event first; return it to let the next area and core handle it too.
pub fn on_event(_a: &mut SlintApp, ev: Event) -> Option<Event> {
    Some(ev)
}
