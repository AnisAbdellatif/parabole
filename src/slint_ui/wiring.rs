//! Cross-area wiring: the browse area asks for dialogs, the dialogs area owns them.
use super::state::with_app;
use super::{AppWindow, BrowseApi, DialogApi};
use slint::ComponentHandle;

pub fn connect(win: &AppWindow) {
    let weak = win.as_weak();
    win.global::<BrowseApi>().on_request_dialog(move |which| {
        let Some(w) = weak.upgrade() else {
            return;
        };
        let dialogs = w.global::<DialogApi>();
        if which == "group" {
            let browse = w.global::<BrowseApi>();
            let add = with_app(|a| a.browse.group_ctx.take().and_then(|c| c.add)).flatten();
            let (source, key) = add.map_or((-1, String::new()), |(s, k)| (s as i32, k));
            dialogs.set_group_id(browse.get_group_ctx_id());
            dialogs.set_group_name(browse.get_group_ctx_name());
            dialogs.set_group_add_source(source);
            dialogs.set_group_add_key(key.into());
        }
        dialogs.invoke_open_dialog(which);
    });
}
