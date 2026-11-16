//! Pure text/number helpers for guide data.
use crate::model::{NowNext, Programme};
use chrono::{Local, TimeZone};

pub fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64)
}

/// Local "HH:MM".
pub fn hm(t: i64) -> String {
    Local
        .timestamp_opt(t, 0)
        .single()
        .map_or_else(|| "--:--".into(), |d| d.format("%H:%M").to_string())
}

/// Elapsed share of the programme at `now`, clamped to 0..=1.
pub fn fraction(p: &Programme, now: i64) -> f32 {
    if p.stop <= p.start {
        return 0.0;
    }
    ((now - p.start) as f32 / (p.stop - p.start) as f32).clamp(0.0, 1.0)
}

pub fn now_line(n: &NowNext) -> String {
    let p = &n.now;
    format!("{} · {}–{}", p.title, hm(p.start), hm(p.stop))
}

pub fn remaining(p: &Programme, now: i64) -> String {
    let mins = ((p.stop - now).max(0) + 59) / 60;
    let mins = mins.max(1);
    if mins >= 60 {
        format!("{} h {} min left", mins / 60, mins % 60)
    } else {
        format!("{mins} min left")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prog(start: i64, stop: i64) -> Programme {
        Programme {
            channel: "c".into(),
            start,
            stop,
            title: "News".into(),
            descr: None,
        }
    }

    #[test]
    fn hm_has_clock_shape() {
        let s = hm(1_700_000_000);
        assert_eq!(s.len(), 5);
        assert_eq!(&s[2..3], ":");
    }

    #[test]
    fn fraction_clamps() {
        let p = prog(100, 200);
        assert_eq!(fraction(&p, 50), 0.0);
        assert_eq!(fraction(&p, 150), 0.5);
        assert_eq!(fraction(&p, 300), 1.0);
        assert_eq!(fraction(&prog(200, 200), 250), 0.0);
    }

    #[test]
    fn remaining_formats() {
        let p = prog(0, 3600 * 2);
        assert_eq!(remaining(&p, 7200 - 61), "2 min left");
        assert_eq!(remaining(&p, 7200 - 1), "1 min left");
        assert_eq!(remaining(&p, 7200), "1 min left");
        assert_eq!(remaining(&p, 0), "2 h 0 min left");
        assert_eq!(remaining(&p, 600), "1 h 50 min left");
    }

    #[test]
    fn now_line_structure() {
        let n = NowNext {
            now: prog(0, 60),
            next: None,
        };
        let s = now_line(&n);
        assert!(s.starts_with("News · "));
        assert!(s.contains('–'));
    }
}
