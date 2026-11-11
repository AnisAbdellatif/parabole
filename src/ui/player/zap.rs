//! Channel-number zapping: digits typed over the video build a number that plays after a pause.

use super::App;
use crate::model::{MediaKind, StreamRow};
use crate::worker::Command;
use egui::{Key, Modifiers};
use std::time::{Duration, Instant};

const IDLE: Duration = Duration::from_millis(1500);
const MAX_DIGITS: usize = 5;
const DIGIT_KEYS: [(Key, char); 10] = [
    (Key::Num0, '0'),
    (Key::Num1, '1'),
    (Key::Num2, '2'),
    (Key::Num3, '3'),
    (Key::Num4, '4'),
    (Key::Num5, '5'),
    (Key::Num6, '6'),
    (Key::Num7, '7'),
    (Key::Num8, '8'),
    (Key::Num9, '9'),
];

#[derive(Default)]
pub struct ZapBuffer {
    digits: String,
    last: Option<Instant>,
}

impl ZapBuffer {
    pub fn push(&mut self, digit: char, now: Instant) {
        if self.digits.len() < MAX_DIGITS {
            self.digits.push(digit);
        }
        self.last = Some(now);
    }

    pub fn text(&self) -> Option<&str> {
        (!self.digits.is_empty()).then_some(self.digits.as_str())
    }

    pub fn cancel(&mut self) {
        self.digits.clear();
        self.last = None;
    }

    /// Takes the typed number now (Enter).
    pub fn take(&mut self) -> Option<i64> {
        let n = self.digits.parse().ok();
        self.cancel();
        n
    }

    /// Takes the typed number once nothing was typed for the idle timeout.
    pub fn take_if_idle(&mut self, now: Instant) -> Option<i64> {
        let idle = self.last.is_some_and(|t| now.duration_since(t) >= IDLE);
        idle.then(|| self.take()).flatten()
    }

    pub fn time_left(&self, now: Instant) -> Option<Duration> {
        self.last
            .map(|t| IDLE.saturating_sub(now.duration_since(t)))
    }
}

/// Live row for channel number `n`: by its number, or by 1-based position when no row has one.
fn row_for_number(rows: &[StreamRow], n: i64) -> Option<usize> {
    let at = if rows.iter().any(|r| r.num.is_some()) {
        rows.iter().position(|r| r.num == Some(n))?
    } else {
        usize::try_from(n.checked_sub(1)?).ok()?
    };
    (rows.get(at)?.media == MediaKind::Live).then_some(at)
}

impl App {
    pub(super) fn zap_keys(&mut self, ctx: &egui::Context, focused: bool) {
        let now = Instant::now();
        let mut ready = None;
        if focused {
            let zap = &mut self.player_ui.zap;
            for (key, digit) in DIGIT_KEYS {
                if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, key)) {
                    zap.push(digit, now);
                }
            }
            if zap.text().is_some() {
                if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
                    zap.cancel();
                } else if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Enter)) {
                    ready = zap.take();
                }
            }
        }
        ready = ready.or_else(|| self.player_ui.zap.take_if_idle(now));
        if let Some(left) = self.player_ui.zap.time_left(now) {
            ctx.request_repaint_after(left + Duration::from_millis(20));
        }
        if let Some(n) = ready {
            self.zap_to_number(n);
        }
    }

    fn zap_to_number(&mut self, n: i64) {
        let Some(i) = row_for_number(&self.streams, n) else {
            self.toast(format!("No live channel {n}"));
            return;
        };
        let id = self.streams[i].id;
        self.selected = i;
        self.scroll_to_selected = true;
        self.worker.send(Command::Play {
            profile: self.profile(),
            stream_id: id,
            kind: Some(crate::player::PlayerKind::Embedded),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(id: i64, num: Option<i64>, media: MediaKind) -> StreamRow {
        StreamRow {
            id,
            source_id: 1,
            media,
            key: String::new(),
            name: String::new(),
            num,
            logo: None,
            favorite: false,
            hidden: false,
            missing: false,
        }
    }

    #[test]
    fn buffer_fires_after_idle_or_enter() {
        let t0 = Instant::now();
        let mut b = ZapBuffer::default();
        b.push('1', t0);
        b.push('2', t0 + Duration::from_millis(1000));
        assert_eq!(b.text(), Some("12"));
        assert_eq!(b.take_if_idle(t0 + Duration::from_millis(2000)), None);
        assert_eq!(b.take_if_idle(t0 + Duration::from_millis(2600)), Some(12));
        assert_eq!(b.text(), None);
        b.push('7', t0);
        assert_eq!(b.take(), Some(7));
        b.push('9', t0);
        b.cancel();
        assert_eq!(b.take_if_idle(t0 + Duration::from_secs(9)), None);
    }

    #[test]
    fn buffer_caps_digits() {
        let mut b = ZapBuffer::default();
        for _ in 0..9 {
            b.push('1', Instant::now());
        }
        assert_eq!(b.text(), Some("11111"));
    }

    #[test]
    fn number_lookup_by_num_or_position() {
        let numbered = [
            row(1, Some(5), MediaKind::Live),
            row(2, Some(9), MediaKind::Live),
        ];
        assert_eq!(row_for_number(&numbered, 9), Some(1));
        assert_eq!(row_for_number(&numbered, 2), None);
        let plain = [
            row(1, None, MediaKind::Live),
            row(2, None, MediaKind::Movie),
        ];
        assert_eq!(row_for_number(&plain, 1), Some(0));
        assert_eq!(row_for_number(&plain, 2), None);
        assert_eq!(row_for_number(&plain, 0), None);
    }
}
