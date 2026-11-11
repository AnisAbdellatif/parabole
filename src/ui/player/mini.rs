//! Mini player: the window shrinks to a small always-on-top video and restores on exit.

use super::App;
use egui::{Pos2, Vec2, ViewportCommand, WindowLevel, vec2};

const MINI_SIZE: Vec2 = vec2(480.0, 270.0);

impl App {
    pub(in crate::ui) fn set_mini(&mut self, ctx: &egui::Context, on: bool) {
        if on == self.player_ui.mini {
            return;
        }
        if on && self.player_ui.fullscreen {
            self.set_fullscreen(ctx, false);
        }
        self.player_ui.mini = on;
        if on {
            ctx.send_viewport_cmd(ViewportCommand::InnerSize(MINI_SIZE));
            ctx.send_viewport_cmd(ViewportCommand::WindowLevel(WindowLevel::AlwaysOnTop));
            return;
        }
        ctx.send_viewport_cmd(ViewportCommand::WindowLevel(WindowLevel::Normal));
        if let Some((size, pos)) = self.player_ui.windowed {
            ctx.send_viewport_cmd(ViewportCommand::InnerSize(size));
            if let Some(pos) = pos {
                ctx.send_viewport_cmd(ViewportCommand::OuterPosition(pos));
            }
        }
    }

    /// Tracks the normal window geometry so mini mode can restore it.
    pub(super) fn remember_window(&mut self, ctx: &egui::Context) {
        if self.player_ui.fullscreen || self.player_ui.mini {
            return;
        }
        let (inner, outer, full) = ctx.input(|i| {
            let v = i.viewport();
            (v.inner_rect, v.outer_rect, v.fullscreen)
        });
        let Some(inner) = inner.filter(|_| full != Some(true)) else {
            return;
        };
        // the viewport still reports the mini size for a frame or two after leaving mini mode
        if (inner.size() - MINI_SIZE).abs().max_elem() < 1.0 {
            return;
        }
        let pos: Option<Pos2> = outer.map(|r| r.min);
        self.player_ui.windowed = Some((inner.size(), pos));
    }
}
