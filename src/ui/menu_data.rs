//! Context the right-click menu needs but cannot reach (it only gets `ui`): published per frame.
use super::App;
use crate::model::{Alternate, MediaKind, StreamGroup};
use egui::{Context, Id, Ui};
use std::sync::Arc;

#[derive(Clone)]
pub struct MenuData {
    pub media: MediaKind,
    pub groups: Vec<StreamGroup>,
    /// Group currently open in the list, if any.
    pub in_group: Option<i64>,
    /// Alternates of the stream with this id, once loaded.
    pub alts: Option<(i64, Vec<Alternate>)>,
}

fn id() -> Id {
    Id::NULL.with("row_menu_data")
}

pub fn current(ui: &Ui) -> Option<Arc<MenuData>> {
    ui.data(|d| d.get_temp::<Arc<MenuData>>(id()))
}

impl App {
    pub(super) fn publish_menu_data(&self, ctx: &Context) {
        let in_group = match self.state.sel {
            Some(super::state::Sel::Group(g)) if self.search.applied.is_empty() => Some(g),
            _ => None,
        };
        let data = MenuData {
            media: self.state.tab,
            groups: self.groups.clone(),
            in_group,
            alts: self.alts.clone(),
        };
        ctx.data_mut(|d| d.insert_temp(id(), Arc::new(data)));
    }
}
