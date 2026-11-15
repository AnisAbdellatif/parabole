//! Group dialog (mirrors `ui/group_dialog.rs` and `save_group` in `ui/group_actions.rs`).
use super::{AppWindow, DialogApi};
use crate::slint_ui::state::{SlintApp, with_app};
use slint::ComponentHandle;

impl SlintApp {
    fn save_group_from_dialog(&mut self) {
        let Some(w) = self.win.upgrade() else { return };
        let api = w.global::<DialogApi>();
        let name = api.get_group_name().trim().to_string();
        if name.is_empty() {
            return;
        }
        let (id, source) = (api.get_group_id(), api.get_group_add_source());
        let add = (source >= 0).then(|| (i64::from(source), api.get_group_add_key().to_string()));
        let profile = self.profile();
        let res = if id >= 0 {
            self.db.rename_group(id.into(), &name)
        } else {
            self.db
                .create_group(profile, self.state.tab, &name)
                .and_then(|g| match add {
                    Some((source, key)) => self.db.add_to_group(g, source, &key),
                    None => Ok(()),
                })
        };
        self.report(res);
        self.close_dialog();
        self.reload_categories();
    }
}

pub fn connect(win: &AppWindow) {
    win.global::<DialogApi>().on_group_save(|| {
        with_app(|a| a.save_group_from_dialog());
    });
}
