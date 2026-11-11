//! Background thread for network, DB-write and player jobs. The UI talks to it
//! through `Command`/`Event` channels and never blocks on it.

mod embed;
mod play;
pub mod player_settings;
#[cfg(test)]
mod tests;

pub use embed::{EmbedPlay, Track, episode_key};

use crate::db::Db;
use crate::model::{
    Episode, MediaKind, ParsedPlaylist, PlayRequest, ProfileId, Season, Source, SourceId,
};
use crate::player::{ExternalPlayer, PlayerKind};
use crate::sources;
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, channel};
use std::thread;
use std::time::Duration;

pub type Repaint = Arc<dyn Fn() + Send + Sync>;

#[derive(Debug)]
pub enum Command {
    AddSource(ProfileId, Source),
    UpdateSource(Source),
    DeleteSource(SourceId),
    Refresh(SourceId),
    RefreshAll(ProfileId),
    LoadSeries {
        source: SourceId,
        key: String,
    },
    Play {
        profile: ProfileId,
        stream_id: i64,
        kind: Option<PlayerKind>,
    },
    PlayEpisode {
        profile: ProfileId,
        source: SourceId,
        episode: Episode,
        title: String,
        kind: Option<PlayerKind>,
    },
    CopyUrl {
        stream_id: i64,
    },
    /// Hand an already-resolved request to a player (embedded-player fallback).
    LaunchRequest {
        req: PlayRequest,
        kind: PlayerKind,
        track: Option<Track>,
        stream_id: Option<i64>,
    },
    SavePosition {
        track: Track,
        position_s: f64,
        duration_s: f64,
    },
    /// Reload the external mpv stream, landing on the live edge.
    GoLive,
    StopPlayer,
    Fetched(SourceId, Result<ParsedPlaylist, String>),
    Quit,
}

#[derive(Debug)]
pub enum Event {
    SourceChanged,
    RefreshStarted(SourceId),
    RefreshDone(SourceId, Result<(), String>),
    SeriesLoaded {
        source: SourceId,
        key: String,
        result: Result<Vec<Season>, String>,
    },
    Playing {
        title: String,
        kind: PlayerKind,
        media: MediaKind,
    },
    PlayEmbedded(Box<EmbedPlay>),
    PlayerStopped,
    PlayFailed(String),
    UrlReady(String),
    Error(String),
}

pub struct Handle {
    pub tx: Sender<Command>,
    pub rx: Receiver<Event>,
}

impl Handle {
    pub fn send(&self, cmd: Command) {
        let _ = self.tx.send(cmd);
    }
}

pub fn spawn(db_path: PathBuf, repaint: Repaint) -> Result<Handle> {
    let db = Db::open(&db_path).context("open worker database")?;
    let (tx, cmd_rx) = channel();
    let (ev_tx, rx) = channel();
    let mut worker = Worker {
        db,
        player: ExternalPlayer::default(),
        cmd_tx: tx.clone(),
        ev_tx,
        repaint,
        was_playing: false,
    };
    thread::Builder::new()
        .name("worker".into())
        .spawn(move || worker.run(cmd_rx))
        .context("spawn worker thread")?;
    Ok(Handle { tx, rx })
}

struct Worker {
    db: Db,
    player: ExternalPlayer,
    cmd_tx: Sender<Command>,
    ev_tx: Sender<Event>,
    repaint: Repaint,
    was_playing: bool,
}

impl Worker {
    fn emit(&self, ev: Event) {
        let _ = self.ev_tx.send(ev);
        (self.repaint)();
    }

    fn run(&mut self, rx: Receiver<Command>) {
        loop {
            match rx.recv_timeout(Duration::from_secs(1)) {
                Ok(Command::Quit) | Err(RecvTimeoutError::Disconnected) => break,
                Ok(cmd) => self.handle(cmd),
                Err(RecvTimeoutError::Timeout) => {}
            }
            let playing = self.player.is_running();
            if self.was_playing && !playing {
                self.emit(Event::PlayerStopped);
            }
            self.was_playing = playing;
        }
        self.player.stop();
    }

    fn handle(&mut self, cmd: Command) {
        let result = match cmd {
            Command::AddSource(profile, src) => self.add_source(profile, &src),
            Command::UpdateSource(src) => self.db.update_source(&src).map(|_| self.changed()),
            Command::DeleteSource(id) => self.db.delete_source(id).map(|_| self.changed()),
            Command::Refresh(id) => self.start_refresh(id),
            Command::RefreshAll(profile) => self.refresh_all(profile),
            Command::LoadSeries { source, key } => self.load_series(source, key),
            Command::Fetched(id, res) => self.apply_fetched(id, res),
            Command::Play {
                profile,
                stream_id,
                kind,
            } => self.play(profile, stream_id, kind.as_ref()),
            Command::PlayEpisode {
                profile,
                source,
                episode,
                title,
                kind,
            } => self.play_episode(profile, source, &episode, title, kind.as_ref()),
            Command::CopyUrl { stream_id } => self.copy_url(stream_id),
            Command::LaunchRequest {
                req,
                kind,
                track,
                stream_id,
            } => self.launch_request(req, kind, track, stream_id),
            Command::SavePosition {
                track,
                position_s,
                duration_s,
            } => self.save_position(&track, position_s, duration_s),
            Command::GoLive => self.player.go_live(),
            Command::StopPlayer => {
                self.player.stop();
                Ok(())
            }
            Command::Quit => Ok(()),
        };
        if let Err(e) = result {
            self.emit(Event::Error(format!("{e:#}")));
        }
    }

    fn changed(&self) {
        self.emit(Event::SourceChanged);
    }

    fn add_source(&mut self, profile: ProfileId, src: &Source) -> Result<()> {
        let id = self.db.add_source(profile, src)?;
        self.changed();
        self.start_refresh(id)
    }

    fn refresh_all(&mut self, profile: ProfileId) -> Result<()> {
        for s in self.db.profile_sources(profile)? {
            self.start_refresh(s.id)?;
        }
        Ok(())
    }

    fn start_refresh(&mut self, id: SourceId) -> Result<()> {
        let source = self.db.source(id)?;
        self.emit(Event::RefreshStarted(id));
        let tx = self.cmd_tx.clone();
        let repaint = self.repaint.clone();
        thread::spawn(move || {
            let res = sources::fetch(&source).map_err(|e| format!("{e:#}"));
            let _ = tx.send(Command::Fetched(id, res));
            repaint();
        });
        Ok(())
    }

    fn apply_fetched(&mut self, id: SourceId, res: Result<ParsedPlaylist, String>) -> Result<()> {
        let outcome = match res {
            Ok(data) => self
                .db
                .replace_source_data(id, &data)
                .map_err(|e| format!("{e:#}")),
            Err(e) => Err(e),
        };
        self.db
            .set_refresh_result(id, outcome.as_ref().err().map(String::as_str))?;
        self.emit(Event::RefreshDone(id, outcome));
        Ok(())
    }

    fn load_series(&mut self, source: SourceId, key: String) -> Result<()> {
        let src = self.db.source(source)?;
        let (tx, repaint) = (self.ev_tx.clone(), self.repaint.clone());
        thread::spawn(move || {
            let result = sources::fetch_series(&src, &key).map_err(|e| format!("{e:#}"));
            let _ = tx.send(Event::SeriesLoaded {
                source,
                key,
                result,
            });
            repaint();
        });
        Ok(())
    }
}
