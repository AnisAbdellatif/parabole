//! Settings dialog (mirrors `ui/settings_view.rs`).
use super::{AppWindow, DialogApi};
use crate::player::{PlayerKind, PlayerSettings, find_binary};
use crate::slint_ui::CheckItem;
use crate::slint_ui::state::{SlintApp, with_app};
use crate::sports::lang::LANGUAGES;
use crate::sports::registry::SPORTS;
use crate::worker::player_settings;
use crate::worker::sports_settings::{self, SportsSettings};
use crate::worker::trakt::store;
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel};
use std::path::PathBuf;

fn kind_label(kind: &PlayerKind) -> &'static str {
    match kind {
        PlayerKind::Embedded => "Built-in player",
        PlayerKind::Mpv => "mpv",
        PlayerKind::Vlc => "VLC",
        PlayerKind::Custom => "Custom command",
    }
}

fn kind_names() -> Vec<SharedString> {
    PlayerKind::selectable()
        .iter()
        .map(|k| kind_label(k).into())
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
    pub(super) fn load_settings_dialog(&mut self, tab: i32) {
        self.load_appearance_dialog();
        let Some(w) = self.win.upgrade() else { return };
        let s = player_settings::load(&self.db);
        let f = sports_settings::load(&self.db);
        let api = w.global::<DialogApi>();
        api.set_settings_tab(tab);
        api.set_set_sports(f.enabled);
        api.set_set_imdb(crate::worker::meta::enabled(&self.db));
        let langs: Vec<CheckItem> = LANGUAGES
            .iter()
            .map(|l| CheckItem {
                code: l.code.into(),
                name: l.name.into(),
                checked: f.langs.iter().any(|c| c == l.code),
            })
            .collect();
        api.set_sports_langs(ModelRc::new(VecModel::from(langs)));
        let sports: Vec<CheckItem> = SPORTS
            .iter()
            .map(|s| CheckItem {
                code: s.id.into(),
                name: s.name.into(),
                checked: f.sports.iter().any(|c| c == s.id),
            })
            .collect();
        api.set_sports_list(ModelRc::new(VecModel::from(sports)));
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
        self.push_guides();
        super::trakt::load(self);
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
        let sports = SportsSettings {
            enabled: api.get_set_sports(),
            langs: api
                .get_sports_langs()
                .iter()
                .filter(|l| l.checked)
                .map(|l| l.code.to_string())
                .collect(),
            sports: api
                .get_sports_list()
                .iter()
                .filter(|l| l.checked)
                .map(|l| l.code.to_string())
                .collect(),
        };
        if sports.enabled && sports.sports.is_empty() {
            return Err("Pick at least one sport".into());
        }
        if sports.enabled && sports.langs.is_empty() {
            return Err("Pick at least one language for the Sports tab".into());
        }
        player_settings::save(&self.db, &s).map_err(|e| format!("{e:#}"))?;
        sports_settings::save(&self.db, &sports).map_err(|e| format!("{e:#}"))?;
        crate::slint_ui::sports::apply_settings(self, sports);
        let imdb = api.get_set_imdb();
        if imdb != crate::worker::meta::enabled(&self.db) {
            crate::worker::meta::set_enabled(&self.db, imdb).map_err(|e| format!("{e:#}"))?;
            // forget lookups made under the old setting so the next open asks again
            self.browse.info.meta.clear();
        }
        let profile = self.profile();
        store::set_scrobble_enabled(&self.db, profile, api.get_set_trakt_scrobble())
            .map_err(|e| format!("{e:#}"))?;
        Ok(())
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
    api.on_sport_toggled(|i| {
        with_app(|a| {
            let Some(w) = a.win.upgrade() else { return };
            let model = w.global::<DialogApi>().get_sports_list();
            let Ok(i) = usize::try_from(i) else { return };
            if let Some(mut l) = model.row_data(i) {
                l.checked = !l.checked;
                model.set_row_data(i, l);
            }
        });
    });
    api.on_sports_lang_toggled(|i| {
        with_app(|a| {
            let Some(w) = a.win.upgrade() else { return };
            let model = w.global::<DialogApi>().get_sports_langs();
            let Ok(i) = usize::try_from(i) else { return };
            if let Some(mut l) = model.row_data(i) {
                l.checked = !l.checked;
                model.set_row_data(i, l);
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
