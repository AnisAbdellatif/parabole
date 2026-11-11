//! egui views. The UI never does network I/O: it sends `Command`s to the worker.

mod boot;
mod data;
mod details_view;
mod edit_mode;
mod events;
mod hide_new;
mod player;
mod profiles_view;
mod series_view;
mod settings_view;
mod sidebar;
mod sort;
mod sources;
mod state;
mod statusbar;
mod stream_list;
mod stream_row;
mod theme;
mod toasts;
mod topbar;

pub use boot::run;

use crate::db::Db;
use crate::model::{CategoryRow, Profile, Source, SourceId, StreamRow};
use crate::player::{PlayerKind, PlayerSettings};
use crate::sources::xtream::AccountInfo;
use crate::worker::Handle;
use state::{EditState, Persisted, SearchState, SeriesState, Toast};
use std::collections::{HashMap, HashSet};

pub struct App {
    db: Db,
    worker: Handle,
    state: Persisted,
    profiles: Vec<Profile>,
    sources: Vec<Source>,
    categories: Vec<CategoryRow>,
    streams: Vec<StreamRow>,
    selected: usize,
    sort: sort::SortMode,
    scroll_to_selected: bool,
    list_view: (f32, f32),
    search: SearchState,
    edit: EditState,
    series: Option<SeriesState>,
    refreshing: HashSet<SourceId>,
    accounts: HashMap<SourceId, AccountInfo>,
    info: details_view::InfoStore,
    playing: Option<(String, PlayerKind)>,
    player_ui: player::PlayerUi,
    player_settings: PlayerSettings,
    toasts: Vec<Toast>,
    sources_dialog: Option<sources::SourcesDialog>,
    settings_dialog: Option<settings_view::SettingsDialog>,
    profiles_dialog: Option<profiles_view::ProfilesDialog>,
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.pump_events(&ctx);
        self.apply_search_if_due(&ctx);
        self.embed_tick(&ctx);
        if !self.embed_fullscreen(ui, frame) {
            self.topbar(ui);
            self.status_bar(ui);
            self.sidebar(ui);
            self.embed_pane(ui, frame);
            egui::CentralPanel::default().show(ui, |ui| self.center(ui));
        }
        self.dialogs(&ctx);
        self.toasts(&ctx);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, "ui", &self.state);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.shutdown_embedded();
        self.worker.send(crate::worker::Command::Quit);
    }
}

impl App {
    fn dialogs(&mut self, ctx: &egui::Context) {
        self.sources_window(ctx);
        self.settings_window(ctx);
        self.profiles_window(ctx);
        self.details_window(ctx);
    }
}
