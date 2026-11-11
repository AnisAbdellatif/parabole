//! When a live stream gives up for good, try the same channel from another source once.
use super::App;
use crate::worker::Command;

impl App {
    /// `failed` = id of the live stream that gave up. A stream we switched *to* never switches again,
    /// so two dead sources cannot ping-pong. shortcut: the guard lasts until another stream gives up.
    pub(super) fn switch_to_alternate(&mut self, failed: i64) {
        if self.player_ui.switched_to == Some(failed) {
            self.player_ui.switched_to = None;
            return;
        }
        let alt = match self.db.alternates(self.profile(), failed) {
            Ok(alts) => alts.into_iter().next(),
            Err(e) => return self.toast(format!("{e:#}")),
        };
        let Some(alt) = alt else {
            return;
        };
        self.player_ui.switched_to = Some(alt.stream_id);
        self.toast(format!("Switched to {}", alt.source_name));
        self.worker.send(Command::Play {
            profile: self.profile(),
            stream_id: alt.stream_id,
            kind: None,
        });
    }
}
