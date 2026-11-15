//! Channel-number zapping: digits typed over the video build a number that plays after a pause.

use std::time::{Duration, Instant};

const IDLE: Duration = Duration::from_millis(1500);
const MAX_DIGITS: usize = 5;
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
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
