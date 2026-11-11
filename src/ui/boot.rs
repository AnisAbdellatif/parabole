use super::App;
use super::player::PlayerUi;
use super::state::Persisted;
use crate::db::Db;
use crate::player::{PlayerKind, live};
use crate::worker::{self, player_settings};
use anyhow::{Context, Result};
use directories::ProjectDirs;
use std::path::PathBuf;
use std::sync::Arc;

fn db_path() -> Result<PathBuf> {
    let dirs =
        ProjectDirs::from("dev", "iptv-player", "IPTV Player").context("no home directory")?;
    std::fs::create_dir_all(dirs.data_dir()).context("create data dir")?;
    Ok(dirs.data_dir().join("iptv.db"))
}

fn install_mpv_script(db: &std::path::Path) {
    if let Some(dir) = db.parent() {
        let _ = live::install_script(dir);
    }
}

pub fn run() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 800.0]),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native(
        "IPTV Player",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(App::new(cc)?))
        }),
    )
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Result<Self> {
        let path = db_path()?;
        install_mpv_script(&path);
        let db = Db::open(&path).context("open database")?;
        let ctx = cc.egui_ctx.clone();
        let worker = worker::spawn(path, Arc::new(move || ctx.request_repaint()))?;
        let state: Persisted = cc
            .storage
            .and_then(|s| eframe::get_value(s, "ui"))
            .unwrap_or_default();
        let default_profile = db.ensure_default_profile()?;
        let player_settings = player_settings::load(&db);
        let mut app = App {
            db,
            worker,
            state,
            profiles: Vec::new(),
            sources: Vec::new(),
            categories: Vec::new(),
            streams: Vec::new(),
            selected: 0,
            scroll_to_selected: false,
            list_view: (0.0, 0.0),
            search: Default::default(),
            edit: Default::default(),
            series: None,
            refreshing: Default::default(),
            playing: None::<(String, PlayerKind)>,
            player_ui: PlayerUi::new(cc, &player_settings),
            player_settings,
            toasts: Vec::new(),
            sources_dialog: None,
            settings_dialog: None,
            profiles_dialog: None,
        };
        app.state.profile.get_or_insert(default_profile);
        app.reload_all();
        Ok(app)
    }
}
