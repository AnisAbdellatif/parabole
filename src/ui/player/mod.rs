//! Embedded player UI: pane/fullscreen view, overlay controls, retry and resume handling.

mod controls;
mod dock;
mod fullscreen;
mod keys;
mod mini;
mod nerd_stats;
mod pane;
mod session;
mod video;
mod widgets;
mod zap;

use super::App;
use crate::model::MediaKind;
use crate::player::embed::EmbeddedPlayer;
use crate::player::{Notice, PlayerKind, PlayerSettings};
use crate::worker::{Command, EmbedPlay};
use session::{EndAction, SAVE_EVERY, STABLE_AFTER_S, Session, end_action};
use std::time::Instant;

pub struct PlayerUi {
    embed: Option<EmbeddedPlayer>,
    init_error: Option<String>,
    session: Option<Session>,
    fullscreen: bool,
    mini: bool,
    /// Size and position of the normal window, restored when leaving mini mode.
    windowed: Option<(egui::Vec2, Option<egui::Pos2>)>,
    zap: zap::ZapBuffer,
    /// Stats-for-nerds overlay on the video.
    stats: bool,
    last_motion: Instant,
    video_rect: egui::Rect,
    prev_stream: Option<i64>,
    /// Media of the stream an external player is playing.
    ext_media: Option<MediaKind>,
}

impl PlayerUi {
    pub fn new(cc: &eframe::CreationContext<'_>, settings: &PlayerSettings) -> Self {
        let (embed, init_error) = match EmbeddedPlayer::new(cc, settings) {
            Ok(p) => (Some(p), None),
            Err(e) => (None, Some(format!("{e:#}"))),
        };
        Self {
            embed,
            init_error,
            session: None,
            fullscreen: false,
            mini: false,
            windowed: None,
            zap: zap::ZapBuffer::default(),
            stats: false,
            last_motion: Instant::now(),
            video_rect: egui::Rect::NOTHING,
            prev_stream: None,
            ext_media: None,
        }
    }

    pub fn apply_mpv_args(&self, args: &[String]) {
        if let Some(p) = &self.embed {
            p.apply_mpv_args(args);
        }
    }
}

impl App {
    pub(super) fn embedded_active(&self) -> bool {
        self.player_ui.session.is_some()
    }

    /// True while the current stream is a live channel (external mpv or embedded).
    pub(super) fn live_playing(&self) -> bool {
        match (&self.player_ui.session, &self.playing) {
            (Some(s), _) => s.is_live(),
            (None, Some((_, PlayerKind::Mpv))) => self.player_ui.ext_media == Some(MediaKind::Live),
            _ => false,
        }
    }

    pub(super) fn external_started(&mut self, ctx: &egui::Context, media: MediaKind) {
        self.player_ui.ext_media = Some(media);
        if self.embedded_active() {
            self.stop_embedded(ctx);
        }
    }

    pub(super) fn on_embed_play(&mut self, play: EmbedPlay) {
        self.save_position();
        let ui = &mut self.player_ui;
        let result = match ui.embed.as_mut() {
            Some(p) => p.play(&self.player_settings, &play.req),
            None => Err(anyhow::anyhow!(ui.init_error.clone().unwrap_or_default())),
        };
        match result {
            Ok(()) => {
                let old = ui.session.as_ref().and_then(|s| s.stream_id);
                if old.is_some() && old != play.stream_id {
                    ui.prev_stream = old;
                }
                self.playing = Some((play.req.title.clone(), PlayerKind::Embedded));
                ui.session = Some(Session::new(play));
            }
            Err(e) => {
                self.toast(format!("Embedded player unavailable ({e:#}); using mpv"));
                self.worker.send(Command::LaunchRequest {
                    req: play.req,
                    kind: PlayerKind::Mpv,
                    track: play.track,
                    stream_id: play.stream_id,
                });
            }
        }
    }

    pub(super) fn stop_embedded(&mut self, ctx: &egui::Context) {
        self.save_position();
        if let Some(p) = self.player_ui.embed.as_mut() {
            p.stop();
        }
        self.player_ui.session = None;
        if self.player_ui.fullscreen {
            self.set_fullscreen(ctx, false);
        }
        self.set_mini(ctx, false);
        if matches!(self.playing, Some((_, PlayerKind::Embedded))) {
            self.playing = None;
        }
    }

    pub(super) fn shutdown_embedded(&mut self) {
        self.save_position();
        self.player_ui.embed = None;
    }

    pub(super) fn set_fullscreen(&mut self, ctx: &egui::Context, on: bool) {
        self.player_ui.fullscreen = on;
        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(on));
    }

    fn save_position(&mut self) {
        let Some(s) = self.player_ui.session.as_mut().filter(|s| !s.is_live()) else {
            return;
        };
        if let (Some(track), Some((pos, dur))) = (s.track.clone(), s.known) {
            self.worker.send(Command::SavePosition {
                track,
                position_s: pos,
                duration_s: dur,
            });
            s.last_save = Instant::now();
        }
    }

    /// Per-frame housekeeping: mpv events, live retry, periodic resume saves, key shortcuts.
    pub(super) fn embed_tick(&mut self, ctx: &egui::Context) {
        self.remember_window(ctx);
        self.player_keys(ctx);
        let ui = &mut self.player_ui;
        let (Some(player), Some(s)) = (ui.embed.as_mut(), ui.session.as_mut()) else {
            if let Some(p) = ui.embed.as_mut() {
                p.poll();
            }
            return;
        };
        let mut finished = false;
        for notice in player.poll() {
            let Notice::Ended { failed } = notice else {
                continue;
            };
            match end_action(s.req.media, s.attempt, failed) {
                EndAction::Retry(d) => {
                    s.attempt += 1;
                    s.retry_at = Some(Instant::now() + d);
                    ctx.request_repaint_after(d);
                }
                EndAction::GiveUp => {
                    s.error = Some("The stream stopped and could not be reopened".into())
                }
                EndAction::Finish => finished = true,
            }
        }
        let st = player.state();
        if let Some(pos) = st.time_pos {
            if pos > STABLE_AFTER_S {
                s.attempt = 0;
                s.error = None;
            }
            if let Some(dur) = st.duration {
                s.known = Some((pos, dur));
            }
        }
        if let Some(at) = s.retry_at {
            if Instant::now() >= at {
                s.retry_at = None;
                if let Err(e) = player.play(&self.player_settings, &s.req) {
                    s.error = Some(format!("{e:#}"));
                }
            } else {
                ctx.request_repaint_after(at - Instant::now());
            }
        }
        if finished {
            if let (Some(track), Some((_, dur))) = (s.track.clone(), s.known) {
                self.worker.send(Command::SavePosition {
                    track,
                    position_s: dur,
                    duration_s: dur,
                });
            }
            self.stop_embedded(ctx);
            return;
        }
        if !s.is_live() {
            if s.last_save.elapsed() >= SAVE_EVERY {
                self.save_position();
            }
            ctx.request_repaint_after(SAVE_EVERY);
        }
    }
}
