use super::App;
use crate::model::{MediaKind, StreamRow};
use crate::player::PlayerKind;
use crate::worker::Command;
use egui::{Key, Modifiers};

impl App {
    /// Next row `step` away from the playing one in the current list (wraps, skips series rows).
    pub(super) fn neighbor_row(&self, step: isize) -> Option<&StreamRow> {
        let playing = self.player_ui.session.as_ref()?.stream_id?;
        let n = self.streams.len() as isize;
        let at = self.streams.iter().position(|r| r.id == playing)? as isize;
        (1..=n)
            .map(|d| (at + step * d).rem_euclid(n) as usize)
            .map(|i| &self.streams[i])
            .find(|r| r.media != MediaKind::Series)
    }

    fn play_embedded(&mut self, stream_id: i64) {
        self.worker.send(Command::Play {
            profile: self.profile(),
            stream_id,
            kind: Some(PlayerKind::Embedded),
        });
    }

    fn zap(&mut self, step: isize) {
        let Some(row) = self.neighbor_row(step) else {
            return;
        };
        let id = row.id;
        if let Some(i) = self.streams.iter().position(|r| r.id == id) {
            self.selected = i;
            self.scroll_to_selected = true;
        }
        self.play_embedded(id);
    }

    pub(in crate::ui) fn go_live(&mut self) {
        if let (Some(p), Some(s)) = (
            self.player_ui.embed.as_mut(),
            self.player_ui.session.as_mut(),
        ) {
            if s.is_live() && p.reload(&self.player_settings).is_ok() {
                s.attempt = 0;
                s.error = None;
            }
        } else if self.live_playing() {
            self.worker.send(Command::GoLive);
        }
    }

    pub(super) fn player_keys(&mut self, ctx: &egui::Context) {
        if ctx.egui_wants_keyboard_input() {
            return;
        }
        let key = |m: Modifiers, k: Key| ctx.input_mut(|i| i.consume_key(m, k));
        if key(Modifiers::CTRL, Key::L) {
            self.go_live();
        }
        if !self.embedded_active() {
            return;
        }
        let ui = &self.player_ui;
        let focused = ui.fullscreen
            || ctx
                .pointer_hover_pos()
                .is_some_and(|p| ui.video_rect.contains(p));
        if key(Modifiers::CTRL, Key::ArrowUp) || (focused && key(Modifiers::NONE, Key::PageUp)) {
            self.zap(-1);
        }
        if key(Modifiers::CTRL, Key::ArrowDown) || (focused && key(Modifiers::NONE, Key::PageDown))
        {
            self.zap(1);
        }
        if (key(Modifiers::NONE, Key::Backspace) || key(Modifiers::NONE, Key::L))
            && let Some(prev) = self.player_ui.prev_stream
        {
            self.play_embedded(prev);
        }
        if self.player_ui.fullscreen && key(Modifiers::NONE, Key::Escape) {
            self.set_fullscreen(ctx, false);
        }
        if focused && key(Modifiers::NONE, Key::I) {
            self.player_ui.stats = !self.player_ui.stats;
        }
        if focused && key(Modifiers::NONE, Key::F) {
            let on = !self.player_ui.fullscreen;
            self.set_fullscreen(ctx, on);
        }
        if focused
            && key(Modifiers::NONE, Key::Space)
            && let Some(p) = self.player_ui.embed.as_mut()
        {
            let paused = p.state().paused;
            p.set_paused(!paused);
        }
    }
}
