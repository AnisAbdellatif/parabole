//! Group actions (ports `ui/group_actions.rs`).
use crate::model::{SourceId, StreamRow};
use crate::slint_ui::BrowseApi;
use crate::slint_ui::state::{Sel, SlintApp};
use slint::ComponentHandle;
use std::time::Duration;

/// Row to add to the group the dialog creates; read by `slint_ui::wiring` when it opens the dialog.
pub struct GroupCtx {
    pub add: Option<(SourceId, String)>,
}

pub fn refresh_groups(a: &mut SlintApp) {
    match a.db.groups(a.profile(), a.state.tab) {
        Ok(g) => a.groups = g,
        Err(e) => a.toast(format!("{e:#}")),
    }
    a.push_sidebar();
}

pub fn add_to_group(a: &mut SlintApp, group: i64, row: &StreamRow) {
    let res = a.db.add_to_group(group, row.source_id, &row.key);
    a.report(res);
    refresh_groups(a);
}

pub fn remove_from_group(a: &mut SlintApp, row: &StreamRow) {
    if let Some(Sel::Group(g)) = a.state.sel {
        let res = a.db.remove_from_group(g, row.source_id, &row.key);
        a.report(res);
        refresh_groups(a);
        a.reload_streams();
    }
}

pub fn delete_group(a: &mut SlintApp, id: i64) {
    let res = a.db.delete_group(id);
    a.report(res);
    a.reload_categories();
}

/// Stores the dialog context and asks the dialog layer (wired at merge) to open "group".
pub fn open_group_dialog(
    a: &mut SlintApp,
    id: Option<i64>,
    name: String,
    add: Option<(SourceId, String)>,
) {
    a.browse.group_ctx = Some(GroupCtx { add });
    if let Some(w) = a.win.upgrade() {
        let api = w.global::<BrowseApi>();
        api.set_group_ctx_id(id.map_or(-1, |i| i as i32));
        api.set_group_ctx_name(name.into());
    }
    request_dialog(a, "group");
}

/// Deferred: callbacks must not fire from inside `with_app`.
pub fn request_dialog(a: &SlintApp, which: &'static str) {
    let win = a.win.clone();
    slint::Timer::single_shot(Duration::ZERO, move || {
        if let Some(w) = win.upgrade() {
            w.global::<BrowseApi>().invoke_request_dialog(which.into());
        }
    });
}
