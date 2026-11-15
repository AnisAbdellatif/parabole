//! Browse area. Ownership and spec: docs/slint-phase2.md.
mod edit;
mod filters;
mod groups;
mod menu;
mod rows;
mod series;
mod sidebar;

use super::state::{Sel, SlintApp, with_app};
use super::{AppWindow, BrowseApi, EditCat, EditGroup, SideRow};
use crate::ui::search_filter::SearchFilter;
use crate::ui::sort::SortMode;
use crate::worker::Event;
pub use filters::load_sort;
pub use rows::{RowAction, run as row_action};
use slint::{ComponentHandle, ModelRc, VecModel};
use std::rc::Rc;

/// Per-area UI-thread state, stored in `SlintApp`.
#[derive(Default)]
pub struct BrowseArea {
    pub sort: SortMode,
    pub filter: SearchFilter,
    pub edit: bool,
    pub edit_filter: String,
    pub confirm_group: Option<i64>,
    pub side_rows: Rc<VecModel<SideRow>>,
    pub edit_cats: Rc<VecModel<EditCat>>,
    pub edit_groups: Rc<VecModel<EditGroup>>,
    pub menu_sel: Option<Sel>,
    pub menu_row: Option<usize>,
    pub group_ctx: Option<groups::GroupCtx>,
    clipboard: Option<arboard::Clipboard>,
}

/// Registers this area's `BrowseApi` callbacks. Called once from `run()`.
pub fn connect(win: &AppWindow) {
    let api = win.global::<BrowseApi>();
    with_app(|a| {
        api.set_side_rows(ModelRc::from(a.browse.side_rows.clone()));
        api.set_edit_cats(ModelRc::from(a.browse.edit_cats.clone()));
        api.set_edit_groups(ModelRc::from(a.browse.edit_groups.clone()));
    });
    edit::connect(win);
    sidebar::connect(win);
    filters::connect(win);
    groups::connect(win);
    menu::connect(win);
}

pub fn close_series(a: &mut SlintApp) {
    series::close(a);
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
