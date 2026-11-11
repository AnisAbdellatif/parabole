//! egui views. The UI never does network I/O: it sends `Command`s to the worker.

mod boot;
mod data;
mod details_view;
mod edit_groups;
mod edit_mode;
mod edit_toolbar;
mod events;
mod group_actions;
mod group_dialog;
mod hide_new;
mod kit;
mod list_header;
mod menu_data;
mod player;
mod poster_grid;
mod profiles_view;
mod row_actions;
mod row_menu;
mod search_filter;
mod series_view;
mod settings_view;
mod sidebar;
mod sidebar_groups;
mod sidebar_rows;
mod sort;
mod sources;
mod state;
mod statusbar;
mod stream_list;
mod stream_row;
mod theme;
mod tile;
mod toasts;
mod topbar;

pub use boot::run;

use crate::db::Db;
use crate::model::{Alternate, CategoryRow, Profile, Source, SourceId, StreamGroup, StreamRow};
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
    groups: Vec<StreamGroup>,
    alts: Option<(i64, Vec<Alternate>)>,
    group_dialog: Option<group_dialog::GroupDialog>,
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
            egui::CentralPanel::default()
                .frame(egui::Frame::NONE.fill(theme::BG))
                .show(ui, |ui| self.center(ui));
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
        self.group_window(ctx);
        self.details_window(ctx);
    }
}
