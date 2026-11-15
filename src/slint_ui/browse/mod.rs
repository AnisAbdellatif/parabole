//! Browse area. Ownership and spec: docs/slint-phase2.md.
mod filters;
mod groups;
mod menu;
mod rows;
mod series;

use super::AppWindow;
use super::state::SlintApp;
use crate::ui::search_filter::SearchFilter;
use crate::ui::sort::SortMode;
use crate::worker::Event;
pub use filters::load_sort;
pub use rows::{RowAction, run as row_action};

/// Per-area UI-thread state, stored in `SlintApp`.
#[derive(Default)]
pub struct BrowseArea {
    pub sort: SortMode,
    pub filter: SearchFilter,
    pub edit: bool,
    pub menu_row: Option<usize>,
    pub group_ctx: Option<groups::GroupCtx>,
    clipboard: Option<arboard::Clipboard>,
}

/// Registers this area's `BrowseApi` callbacks. Called once from `run()`.
pub fn connect(win: &AppWindow) {
    filters::connect(win);
    groups::connect(win);
    menu::connect(win);
}

pub fn activate(a: &mut SlintApp, idx: usize) {
    rows::run(a, idx, RowAction::Activate);
}

/// Sees every worker event first; return it to let the next area and core handle it too.
pub fn on_event(a: &mut SlintApp, ev: Event) -> Option<Event> {
    match ev {
        Event::UrlReady(url) => {
            copy_url(a, &url);
            None
        }
        ev => Some(ev),
    }
}

fn copy_url(a: &mut SlintApp, url: &str) {
    if a.browse.clipboard.is_none() {
        a.browse.clipboard = arboard::Clipboard::new().ok();
    }
    let ok = a
        .browse
        .clipboard
        .as_mut()
        .is_some_and(|c| c.set_text(url).is_ok());
    a.toast(if ok {
        "Stream URL copied"
    } else {
        "Could not copy to the clipboard"
    });
}
