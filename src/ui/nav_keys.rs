//! App-level keyboard shortcuts (player keys live in player/keys.rs). Cmd = Ctrl off macOS.
//! - Cmd+1/2/3 switch tab, Cmd+F search, Cmd+D toggle favorite on the selected row.
//! - Cmd+Left focuses the sidebar (Up/Down pick a category, Enter/Right/Cmd+Right back to the list).
//! - List: arrows, PageUp/PageDown, Home/End, Enter (stream_list.rs).
//! - Esc closes the top dialog, else leaves the series page, else clears the search.
use super::App;
use super::state::Sel;
use super::stream_row::RowAction;
use crate::model::MediaKind;
use egui::{Context, Key, Modifiers};

/// Which pane the arrow keys drive, plus sidebar scroll bookkeeping.
#[derive(Default)]
pub struct Nav {
    pub sidebar: bool,
    pub scroll_sidebar: bool,
    pub sidebar_view: (f32, f32),
}

impl App {
    pub(super) fn nav_keys(&mut self, ctx: &Context) {
        let key = |m: Modifiers, k: Key| ctx.input_mut(|i| i.consume_key(m, k));
        for (k, tab) in [
            (Key::Num1, MediaKind::Live),
            (Key::Num2, MediaKind::Movie),
            (Key::Num3, MediaKind::Series),
        ] {
            if key(Modifiers::COMMAND, k) && self.state.tab != tab {
                self.nav.sidebar = false;
                self.set_tab(tab);
            }
        }
        if ctx.egui_wants_keyboard_input() {
            return;
        }
        if key(Modifiers::COMMAND, Key::ArrowLeft) && !self.sources.is_empty() {
            self.nav.sidebar = true;
        }
        if key(Modifiers::COMMAND, Key::ArrowRight) {
            self.nav.sidebar = false;
        }
        if key(Modifiers::COMMAND, Key::D) && self.series.is_none() && !self.streams.is_empty() {
            self.row_action(self.selected, RowAction::ToggleFavorite);
        }
        if self.nav.sidebar && !self.edit.on {
            self.sidebar_keys(ctx);
        }
        // runs after player_keys, which consumes Esc for fullscreen/mini/zap first
        if !egui::Popup::is_any_open(ctx) && key(Modifiers::NONE, Key::Escape) {
            let _ = self.close_top_dialog() || self.series.take().is_some() || self.clear_search();
        }
    }

    fn sidebar_keys(&mut self, ctx: &Context) {
        let (step, to_list) = ctx.input_mut(|i| {
            let down = i.consume_key(Modifiers::NONE, Key::ArrowDown) as isize;
            let up = i.consume_key(Modifiers::NONE, Key::ArrowUp) as isize;
            let enter = i.consume_key(Modifiers::NONE, Key::Enter);
            let right = i.consume_key(Modifiers::NONE, Key::ArrowRight);
            (down - up, enter || right)
        });
        if to_list {
            self.nav.sidebar = false;
        }
        let order = self.sidebar_order();
        if step == 0 || order.is_empty() {
            return;
        }
        let at = order
            .iter()
            .position(|s| Some(s) == self.state.sel.as_ref());
        let next = at.map_or(0, |i| i.saturating_add_signed(step).min(order.len() - 1));
        if at != Some(next) {
            self.select(order[next].clone());
            self.nav.scroll_sidebar = true;
        }
    }

    /// Selectable sidebar entries top to bottom, matching the normal-mode layout.
    fn sidebar_order(&self) -> Vec<Sel> {
        let mut out = vec![Sel::Favorites, Sel::Recent];
        if self.state.tab != MediaKind::Live {
            out.push(Sel::Continue);
        }
        out.extend(self.groups.iter().map(|g| Sel::Group(g.id)));
        for pinned in [true, false] {
            let cats = self.categories.iter().filter(|c| c.pinned == pinned);
            out.extend(cats.map(|c| Sel::Category(c.source_id, c.key.clone())));
        }
        out
    }

    fn close_top_dialog(&mut self) -> bool {
        self.group_dialog.take().is_some()
            || self.close_details()
            || self.profiles_dialog.take().is_some()
            || self.settings_dialog.take().is_some()
            || self.sources_dialog.take().is_some()
    }

    pub(super) fn clear_search(&mut self) -> bool {
        let any = !self.search.text.is_empty() || !self.search.applied.is_empty();
        if any {
            self.search.text.clear();
            self.search.applied.clear();
            self.search.edited_at = None;
            self.search.filter = Default::default();
            self.selected = 0;
            self.reload_streams();
        }
        any
    }
}
