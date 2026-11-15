//! Settings dialog (mirrors `ui/settings_view.rs`).
use super::{AppWindow, DialogApi};
use crate::player::{PlayerKind, PlayerSettings, find_binary};
use crate::slint_ui::state::{SlintApp, with_app};
use crate::worker::player_settings;
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use std::path::PathBuf;

fn kind_names() -> Vec<SharedString> {
    PlayerKind::selectable()
        .iter()
        .map(|k| format!("{k:?}").into())
        .collect()
}

fn pick(kind: &PlayerKind) -> i32 {
    PlayerKind::selectable()
        .iter()
        .position(|k| k == kind)
        .map_or(0, |i| i as i32)
}

/// 0 is "Default" (None), otherwise the selectable kind at `i - 1`.
fn pick_opt(kind: &Option<PlayerKind>) -> i32 {
    kind.as_ref().map_or(0, |k| pick(k) + 1)
}

fn kind_at(i: i32) -> Option<PlayerKind> {
    PlayerKind::selectable()
        .into_iter()
        .nth(usize::try_from(i).ok()?)
}

fn path_text(p: &Option<PathBuf>) -> SharedString {
    p.as_ref()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default()
        .into()
}

fn opt_path(s: &str) -> Option<PathBuf> {
    let s = s.trim();
    (!s.is_empty()).then(|| PathBuf::from(s))
}

fn detect_text() -> String {
    let defaults = PlayerSettings::default();
    let found = |kind| {
        find_binary(&kind, &defaults).map_or("not found".to_string(), |p| p.display().to_string())
    };
    format!(
        "mpv: {} · VLC: {}",
        found(PlayerKind::Mpv),
        found(PlayerKind::Vlc)
    )
}

impl SlintApp {
    pub(super) fn load_settings_dialog(&mut self) {
        let Some(w) = self.win.upgrade() else { return };
        let s = player_settings::load(&self.db);
        let api = w.global::<DialogApi>();
        let names = kind_names();
        let mut opt = vec![SharedString::from("Default")];
        opt.extend(names.iter().cloned());
        api.set_player_kinds(ModelRc::new(VecModel::from(names)));
        api.set_player_kinds_opt(ModelRc::new(VecModel::from(opt)));
        api.set_set_default(pick(&s.default));
        api.set_set_live(pick_opt(&s.live_player));
        api.set_set_movie(pick_opt(&s.movie_player));
        api.set_set_episode(pick_opt(&s.episode_player));
        api.set_set_low_latency(s.live_low_latency);
        api.set_set_catchup(s.live_catchup);
        api.set_set_mpv_path(path_text(&s.mpv_path));
        api.set_set_vlc_path(path_text(&s.vlc_path));
        api.set_set_template(s.custom_template.as_str().into());
        let args = shlex::try_join(s.mpv_args.iter().map(String::as_str)).unwrap_or_default();
        api.set_set_mpv_args(args.into());
        api.set_set_detected("".into());
    }

    fn save_settings_dialog(&mut self) -> Result<(), String> {
        let w = self.win.upgrade().ok_or("window gone")?;
        let api = w.global::<DialogApi>();
        let mpv_args =
            shlex::split(&api.get_set_mpv_args()).ok_or("Unbalanced quotes in mpv args")?;
        let s = PlayerSettings {
            default: kind_at(api.get_set_default()).unwrap_or(PlayerKind::Mpv),
            live_player: kind_at(api.get_set_live() - 1),
            movie_player: kind_at(api.get_set_movie() - 1),
            episode_player: kind_at(api.get_set_episode() - 1),
            live_low_latency: api.get_set_low_latency(),
            live_catchup: api.get_set_catchup(),
            mpv_path: opt_path(&api.get_set_mpv_path()),
            vlc_path: opt_path(&api.get_set_vlc_path()),
            custom_template: api.get_set_template().trim().to_string(),
            mpv_args,
        };
        player_settings::save(&self.db, &s).map_err(|e| format!("{e:#}"))
    }
}

pub fn connect(win: &AppWindow) {
    let api = win.global::<DialogApi>();
    api.on_settings_detect(|| {
        with_app(|a| {
            if let Some(w) = a.win.upgrade() {
                w.global::<DialogApi>()
                    .set_set_detected(detect_text().into());
            }
        });
    });
    api.on_settings_save(|| {
        with_app(|a| match a.save_settings_dialog() {
            Ok(()) => a.close_dialog(),
            Err(e) => a.toast(e),
        });
    });
}
