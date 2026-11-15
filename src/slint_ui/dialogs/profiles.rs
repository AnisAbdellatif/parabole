//! Profiles dialog (mirrors `ui/profiles_view.rs`).
use super::{AppWindow, DialogApi};
use crate::model::Profile;
use crate::slint_ui::ProfileRow;
use crate::slint_ui::state::{SlintApp, with_app};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};

fn copy_name(profiles: &[Profile], base: &str) -> String {
    let taken = |n: &str| profiles.iter().any(|p| p.name == n);
    let mut name = format!("{base} copy");
    let mut i = 2;
    while taken(&name) {
        name = format!("{base} copy {i}");
        i += 1;
    }
    name
}

impl SlintApp {
    pub(super) fn load_profiles_dialog(&mut self) {
        self.dialogs.renaming = None;
        self.dialogs.confirm_profile = None;
        if let Some(w) = self.win.upgrade() {
            w.global::<DialogApi>().set_profile_new_name("".into());
        }
        self.push_profile_rows();
    }

    fn push_profile_rows(&self) {
        let Some(w) = self.win.upgrade() else { return };
        let rows: Vec<ProfileRow> = self
            .profiles
            .iter()
            .map(|p| {
                let others: Vec<SharedString> = self
                    .profiles
                    .iter()
                    .filter(|o| o.id != p.id)
                    .map(|o| o.name.as_str().into())
                    .collect();
                ProfileRow {
                    id: p.id as i32,
                    name: p.name.as_str().into(),
                    renaming: self.dialogs.renaming == Some(p.id),
                    confirm: self.dialogs.confirm_profile == Some(p.id),
                    can_delete: self.profiles.len() > 1,
                    others: ModelRc::new(VecModel::from(others)),
                }
            })
            .collect();
        w.global::<DialogApi>()
            .set_profile_rows(ModelRc::new(VecModel::from(rows)));
    }

    /// Reloads everything after a profile change, then refreshes the dialog rows.
    fn finish_profile_op(&mut self, res: anyhow::Result<()>) {
        self.report(res);
        self.reload_all();
        self.push_profile_rows();
    }
}

pub fn connect(win: &AppWindow) {
    let api = win.global::<DialogApi>();
    api.on_profile_create(|| {
        with_app(|a| {
            let Some(w) = a.win.upgrade() else { return };
            let name = w
                .global::<DialogApi>()
                .get_profile_new_name()
                .trim()
                .to_string();
            if name.is_empty() {
                return;
            }
            w.global::<DialogApi>().set_profile_new_name("".into());
            let res = a.db.create_profile(&name).map(|_| ());
            a.finish_profile_op(res);
        });
    });
    api.on_profile_rename_start(|id| {
        with_app(|a| {
            let Some(p) = a.profiles.iter().find(|p| p.id == i64::from(id)) else {
                return;
            };
            a.dialogs.renaming = Some(p.id);
            if let Some(w) = a.win.upgrade() {
                w.global::<DialogApi>()
                    .set_profile_rename_text(p.name.as_str().into());
            }
            a.push_profile_rows();
        });
    });
    api.on_profile_rename_save(|id| {
        with_app(|a| {
            let Some(w) = a.win.upgrade() else { return };
            let name = w.global::<DialogApi>().get_profile_rename_text();
            a.dialogs.renaming = None;
            let res = a.db.rename_profile(id.into(), name.trim());
            a.finish_profile_op(res);
        });
    });
    api.on_profile_rename_cancel(|| {
        with_app(|a| {
            a.dialogs.renaming = None;
            a.push_profile_rows();
        });
    });
    api.on_profile_duplicate(|id| {
        with_app(|a| {
            let Some(p) = a.profiles.iter().find(|p| p.id == i64::from(id)) else {
                return;
            };
            let name = copy_name(&a.profiles, &p.name);
            let res = a.db.duplicate_profile(p.id, &name).map(|_| ());
            a.finish_profile_op(res);
        });
    });
    api.on_profile_delete(|id| {
        with_app(|a| {
            if a.dialogs.confirm_profile != Some(id.into()) {
                a.dialogs.confirm_profile = Some(id.into());
                a.push_profile_rows();
                return;
            }
            a.dialogs.confirm_profile = None;
            if a.state.profile == Some(id.into()) {
                a.state.profile = None;
            }
            let res = a.db.delete_profile(id.into());
            a.finish_profile_op(res);
        });
    });
    api.on_profile_delete_cancel(|| {
        with_app(|a| {
            a.dialogs.confirm_profile = None;
            a.push_profile_rows();
        });
    });
    api.on_profile_copy_favorites(|to, idx| {
        with_app(|a| {
            let from = a
                .profiles
                .iter()
                .filter(|o| o.id != i64::from(to))
                .nth(idx as usize)
                .map(|o| o.id);
            let Some(from) = from else { return };
            let res = a.db.copy_favorites(from, to.into()).map(|n| {
                a.toast(format!(
                    "Copied {n} favorite{}",
                    if n == 1 { "" } else { "s" }
                ));
            });
            a.finish_profile_op(res);
        });
    });
}
