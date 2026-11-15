//! Browse area. Ownership and spec: docs/slint-phase2.md.
mod details;
mod edit;
mod empty;
mod filters;
mod grid;
mod groups;
mod menu;
mod rows;
mod series;
mod sidebar;

use super::state::{Sel, SlintApp, with_app};
use super::{AppWindow, BrowseApi, EditCat, EditGroup, EpisodeItem, SideRow};
use crate::ui::search_filter::SearchFilter;
use crate::ui::sort::SortMode;
use crate::worker::Event;
pub use filters::load_sort;
pub use grid::GridModel;
pub use rows::{RowAction, run as row_action};
pub use series::open as open_series;
use slint::{ComponentHandle, ModelRc, VecModel};
use std::cell::Cell;
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
    pub series: Option<series::SeriesState>,
    pub series_timer: slint::Timer,
    pub episodes: Rc<VecModel<EpisodeItem>>,
    pub info: details::InfoStore,
    pub grid: Rc<GridModel>,
    pub cta: Cell<Option<empty::Cta>>,
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
        api.set_episodes(ModelRc::from(a.browse.episodes.clone()));
        a.browse.grid.attach(a.streams.clone());
        a.streams.set_grid(&a.browse.grid);
        api.set_grid_rows(ModelRc::from(a.browse.grid.clone()));
        a.streams.set_image_hook(Rc::new(|url| {
            let url = url.to_owned();
            slint::Timer::single_shot(std::time::Duration::ZERO, move || {
                with_app(|a| details::poster_ready(a, &url));
            });
        }));
    });
    api.on_grid_cols_changed(|n| {
        with_app(|a| a.browse.grid.set_cols(n.max(1) as usize));
    });
    api.on_tile_hide_toggled(|i| {
        with_app(|a| rows::run(a, i as usize, RowAction::ToggleHidden));
    });
    details::connect(win);
    series::connect(win);
    empty::connect(win);
    edit::connect(win);
    sidebar::connect(win);
    filters::connect(win);
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
        Event::InfoLoaded {
            source,
            media,
            key,
            result,
        } => {
            details::loaded(a, source, media, key, result);
            None
        }
        Event::SeriesLoaded {
            source,
            key,
            result,
        } => {
            series::loaded(a, source, &key, result);
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
