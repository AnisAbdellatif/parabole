//! "Next episode" countdown shown on the video when an episode ends by itself.
use super::App;
use super::session::Session;
use crate::ui::{kit, theme};
use crate::worker::Track;
use egui::{Align, Color32, Layout, Rect, RichText, Ui, UiBuilder, vec2};
use std::time::{Duration, Instant};

const COUNTDOWN: Duration = Duration::from_secs(5);

pub struct AutoNext {
    pub episode_id: String,
    pub label: String,
    pub deadline: Instant,
}

pub enum Choice {
    Cancel,
    PlayNow,
}

/// The episode after the one `s` was playing, when it came from the series view.
fn find_next(s: &Session) -> Option<AutoNext> {
    let ctx = s.series.as_ref()?;
    let id = s.track.as_ref()?.key.strip_prefix("ep:")?;
    let (season, ep) = crate::episodes::next(&ctx.seasons, id)?;
    Some(AutoNext {
        episode_id: ep.id.clone(),
        label: crate::episodes::label(season, ep),
        deadline: Instant::now() + COUNTDOWN,
    })
}

pub fn paint(ui: &mut Ui, rect: Rect, n: &AutoNext) -> Option<Choice> {
    ui.painter()
        .rect_filled(rect, 0.0, Color32::from_black_alpha(215));
    let left = n.deadline.saturating_duration_since(Instant::now());
    let mut choice = None;
    let mut inner = ui.new_child(
        UiBuilder::new()
            .max_rect(rect.shrink(12.0))
            .layout(Layout::top_down(Align::Center)),
    );
    inner.add_space(((rect.height() - 24.0 - 112.0) / 2.0).max(0.0));
    inner.label(
        RichText::new("Next episode")
            .font(theme::regular(12.0))
            .color(theme::TEXT_MUTED),
    );
    inner.add(
        egui::Label::new(
            RichText::new(&n.label)
                .font(theme::semibold(15.0))
                .color(theme::TEXT),
        )
        .truncate(),
    );
    inner.label(
        RichText::new(format!("Playing in {}s", left.as_secs() + 1))
            .font(theme::regular(12.0))
            .color(theme::TEXT_MUTED),
    );
    inner.add_space(10.0);
    inner.horizontal(|ui| {
        let (cancel, play) = (
            vec2(kit::pill_width(ui, "Cancel"), 30.0),
            vec2(kit::pill_width(ui, "Play now"), 30.0),
        );
        ui.add_space(((ui.available_width() - cancel.x - play.x - 8.0) / 2.0).max(0.0));
        if kit::pill(ui, "Cancel", cancel, false, true).clicked() {
            choice = Some(Choice::Cancel);
        }
        if kit::pill(ui, "Play now", play, true, true).clicked() {
            choice = Some(Choice::PlayNow);
        }
    });
    choice
}

impl App {
    /// Called when an episode ended by itself; true when a countdown started.
    pub(super) fn begin_autonext(&mut self) -> bool {
        let Some(s) = self.player_ui.session.as_mut() else {
            return false;
        };
        s.autonext = find_next(s);
        s.autonext.is_some()
    }

    pub(super) fn autonext_tick(&mut self, ctx: &egui::Context) {
        let due = self
            .player_ui
            .session
            .as_ref()
            .and_then(|s| s.autonext.as_ref())
            .is_some_and(|n| Instant::now() >= n.deadline);
        if due {
            self.autonext_choice(ctx, Choice::PlayNow);
        }
    }

    pub(super) fn autonext_choice(&mut self, ctx: &egui::Context, choice: Choice) {
        let Some(s) = self.player_ui.session.as_mut() else {
            return;
        };
        let Some(next) = s.autonext.take() else {
            return;
        };
        match choice {
            Choice::Cancel => self.stop_embedded(ctx),
            Choice::PlayNow => {
                let cmd = s.series.as_ref().zip(s.track.as_ref()).and_then(
                    |(
                        series,
                        Track {
                            profile, source, ..
                        },
                    )| {
                        series.play_command(*profile, *source, &next.episode_id)
                    },
                );
                match cmd {
                    Some(cmd) => self.worker.send(cmd),
                    None => self.stop_embedded(ctx),
                }
            }
        }
    }
}
