use super::App;
use super::theme::{primary_button, secondary_button};
use crate::player::{PlayerKind, PlayerSettings, find_binary};
use crate::worker::player_settings;
use std::path::PathBuf;

pub struct SettingsDialog {
    default: PlayerKind,
    live: Option<PlayerKind>,
    movie: Option<PlayerKind>,
    episode: Option<PlayerKind>,
    low_latency: bool,
    catchup: bool,
    mpv_path: String,
    vlc_path: String,
    template: String,
    mpv_args: String,
    detected: Option<String>,
}

fn path_text(p: &Option<PathBuf>) -> String {
    p.as_ref()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn opt_path(s: &str) -> Option<PathBuf> {
    let s = s.trim();
    (!s.is_empty()).then(|| PathBuf::from(s))
}

fn kind_label(k: &Option<PlayerKind>) -> String {
    k.as_ref()
        .map_or("Default".to_string(), |k| format!("{k:?}"))
}

fn kind_row(ui: &mut egui::Ui, label: &str, value: &mut Option<PlayerKind>) {
    ui.label(label);
    egui::ComboBox::from_id_salt(label)
        .selected_text(kind_label(value))
        .show_ui(ui, |ui| {
            ui.selectable_value(value, None, "Default");
            for k in PlayerKind::selectable() {
                let label = format!("{k:?}");
                ui.selectable_value(value, Some(k), label);
            }
        });
    ui.end_row();
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

impl App {
    pub(super) fn open_settings(&mut self) {
        let s = &self.player_settings;
        self.settings_dialog = Some(SettingsDialog {
            default: s.default.clone(),
            live: s.live_player.clone(),
            movie: s.movie_player.clone(),
            episode: s.episode_player.clone(),
            low_latency: s.live_low_latency,
            catchup: s.live_catchup,
            mpv_path: path_text(&s.mpv_path),
            vlc_path: path_text(&s.vlc_path),
            template: s.custom_template.clone(),
            mpv_args: shlex::try_join(s.mpv_args.iter().map(String::as_str)).unwrap_or_default(),
            detected: None,
        });
    }

    pub(super) fn settings_window(&mut self, ctx: &egui::Context) {
        let Some(mut d) = self.settings_dialog.take() else {
            return;
        };
        let mut open = true;
        let mut save = false;
        egui::Window::new("Settings")
            .open(&mut open)
            .default_width(480.0)
            .show(ctx, |ui| {
                egui::Grid::new("settings").num_columns(2).show(ui, |ui| {
                    ui.label("Default player");
                    egui::ComboBox::from_id_salt("default_player")
                        .selected_text(format!("{:?}", d.default))
                        .show_ui(ui, |ui| {
                            for k in PlayerKind::selectable() {
                                let label = format!("{k:?}");
                                ui.selectable_value(&mut d.default, k, label);
                            }
                        });
                    ui.end_row();
                    kind_row(ui, "Live player", &mut d.live);
                    kind_row(ui, "Movie player", &mut d.movie);
                    kind_row(ui, "Episode player", &mut d.episode);
                    ui.label("Live latency");
                    ui.checkbox(&mut d.low_latency, "Low-latency buffering");
                    ui.end_row();
                    ui.label("");
                    ui.checkbox(&mut d.catchup, "Catch up to the live edge");
                    ui.end_row();
                    ui.label("mpv path");
                    ui.text_edit_singleline(&mut d.mpv_path);
                    ui.end_row();
                    ui.label("VLC path");
                    ui.text_edit_singleline(&mut d.vlc_path);
                    ui.end_row();
                    ui.label("Custom command");
                    ui.text_edit_singleline(&mut d.template)
                        .on_hover_text("{url} {title} {user_agent} {referrer}");
                    ui.end_row();
                    ui.label("Extra mpv args");
                    ui.text_edit_singleline(&mut d.mpv_args);
                    ui.end_row();
                });
                ui.horizontal(|ui| {
                    if secondary_button(ui, "Detect").clicked() {
                        d.detected = Some(detect_text());
                    }
                    if let Some(t) = &d.detected {
                        ui.weak(t);
                    }
                });
                save = primary_button(ui, "Save").clicked();
            });
        if save {
            match self.save_player_settings(&d) {
                Ok(()) => open = false,
                Err(e) => self.toast(e),
            }
        }
        if open {
            self.settings_dialog = Some(d);
        }
    }

    fn save_player_settings(&mut self, d: &SettingsDialog) -> Result<(), String> {
        let mpv_args = shlex::split(&d.mpv_args).ok_or("Unbalanced quotes in mpv args")?;
        let s = PlayerSettings {
            default: d.default.clone(),
            live_player: d.live.clone(),
            movie_player: d.movie.clone(),
            episode_player: d.episode.clone(),
            live_low_latency: d.low_latency,
            live_catchup: d.catchup,
            mpv_path: opt_path(&d.mpv_path),
            vlc_path: opt_path(&d.vlc_path),
            custom_template: d.template.trim().to_string(),
            mpv_args,
        };
        player_settings::save(&self.db, &s).map_err(|e| format!("{e:#}"))?;
        self.player_ui.apply_mpv_args(&s.mpv_args);
        self.player_settings = s;
        Ok(())
    }
}
