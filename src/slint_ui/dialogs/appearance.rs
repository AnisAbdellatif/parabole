//! Appearance dialog: edits apply live to `Theme` and are saved to the UI state file.
use super::{AppWindow, DialogApi};
use crate::slint_ui::AppearanceApi;
use crate::slint_ui::appearance::{ACCENTS, Appearance};
use crate::slint_ui::state::{SlintApp, with_app};
use slint::{Color, ComponentHandle, ModelRc, VecModel};

/// Applies `f` to the stored appearance, then applies it live and saves it.
// shortcut: saves on every change (a slider drag writes the small JSON per tick).
pub fn commit(a: &mut SlintApp, f: impl FnOnce(&mut Appearance)) {
    let mut ap = a.state.appearance.clone();
    f(&mut ap);
    a.state.appearance = ap.clamped();
    if let Some(w) = a.win.upgrade() {
        a.state.appearance.apply(&w);
    }
    let res = a.state.save(&a.state_path);
    a.report(res);
}

pub fn connect(win: &AppWindow) {
    win.global::<AppearanceApi>().on_changed(|| {
        with_app(|a| {
            let Some(w) = a.win.upgrade() else { return };
            let api = w.global::<AppearanceApi>();
            commit(a, |s| {
                s.accent = api.get_accent().clamp(0, 255) as u8;
                s.mode = api.get_mode().clamp(0, 255) as u8;
                s.density = api.get_density().clamp(0, 255) as u8;
                s.radius_scale = api.get_radius();
                s.font_scale = api.get_font();
                s.reduce_motion = api.get_reduce_motion();
            });
        });
    });
}

impl SlintApp {
    pub(super) fn load_appearance_dialog(&mut self) {
        let Some(w) = self.win.upgrade() else { return };
        let ap = self.state.appearance.clamped();
        let api = w.global::<AppearanceApi>();
        let swatches: Vec<Color> = ACCENTS
            .iter()
            .map(|(c, _)| Color::from_rgb_u8((c >> 16) as u8, (c >> 8) as u8, *c as u8))
            .collect();
        api.set_swatches(ModelRc::new(VecModel::from(swatches)));
        api.set_accent(ap.accent.into());
        api.set_mode(ap.mode.into());
        api.set_density(ap.density.into());
        api.set_radius(ap.radius_scale);
        api.set_font(ap.font_scale);
        api.set_reduce_motion(ap.reduce_motion);
        let _ = w.global::<DialogApi>();
    }
}
