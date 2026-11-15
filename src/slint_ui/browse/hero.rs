//! Hero banner: tracks the focused row, looks up backdrop/colour/info and pushes `HeroExtra`.
use super::details;
use crate::db::progress::{Watch, classify};
use crate::model::{MediaInfo, MediaKind, StreamRow};
use crate::slint_ui::images::Size;
use crate::slint_ui::state::{SlintApp, with_app};
use crate::slint_ui::{AppWindow, BrowseApi, HeroExtra};
use slint::{ComponentHandle, Timer, TimerMode};
use std::time::Duration;

const SETTLE: Duration = Duration::from_millis(300);

#[derive(Default)]
pub struct HeroState {
    focus: Option<usize>,
    timer: Timer,
}

fn parse_secs(s: &str) -> Option<u64> {
    let mut parts = s.trim().split(':');
    let (mut total, mut n) = (0u64, 0);
    for p in parts.by_ref() {
        total = total * 60 + p.trim().parse::<u64>().ok()?;
        n += 1;
    }
    (n >= 2).then_some(total)
}

fn fmt_hms(secs: u64) -> String {
    format!("{}:{:02}:{:02}", secs / 3600, secs / 60 % 60, secs % 60)
}

fn meta(info: &MediaInfo) -> String {
    let year = info
        .release_date
        .as_deref()
        .and_then(|d| d.get(..4))
        .filter(|y| y.chars().all(|c| c.is_ascii_digit()));
    [year, info.duration.as_deref(), info.rating.as_deref()]
        .into_iter()
        .flatten()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" · ")
}

fn action(row: &StreamRow, info: Option<&MediaInfo>) -> String {
    if row.media == MediaKind::Series {
        return row
            .resume_episode
            .as_ref()
            .map_or("Episodes".into(), |e| format!("Resume {}", e.label));
    }
    if classify(row.progress) != Watch::Partial {
        return "Play".into();
    }
    let at = info
        .and_then(|i| i.duration.as_deref())
        .and_then(parse_secs)
        .zip(row.progress)
        .map(|(d, f)| fmt_hms((d as f64 * f64::from(f)) as u64));
    at.map_or("Resume".into(), |t| format!("Resume {t}"))
}

fn build(a: &SlintApp, idx: usize, row: &StreamRow) -> HeroExtra {
    let mut h = HeroExtra {
        index: idx as i32,
        ..Default::default()
    };
    if row.media == MediaKind::Live {
        let color = row.logo.as_deref().and_then(|u| a.images.color(u));
        h.has_color = color.is_some();
        h.color = color.unwrap_or_default();
        return h;
    }
    let key = (row.source_id, row.media, row.key.clone());
    let info = details::cached(a, &key);
    let url = info
        .and_then(|i| i.backdrop.as_deref().or(i.cover.as_deref()))
        .or(row.logo.as_deref());
    if let Some(img) = url.and_then(|u| a.images.get(u, Size::Backdrop)) {
        h.backdrop = img;
        h.has_backdrop = true;
    }
    if let Some(i) = info {
        h.meta = meta(i).into();
        h.plot = i.plot.clone().unwrap_or_default().into();
    }
    h.action = action(row, info).into();
    h
}

/// Re-pushes the hero for the current focus (image or info arrived, list changed).
pub fn push(a: &SlintApp) {
    let Some(win) = a.win.upgrade() else {
        return;
    };
    let hero = a
        .browse
        .hero
        .focus
        .and_then(|i| a.streams.row(i).map(|r| build(a, i, &r)))
        .unwrap_or(HeroExtra {
            index: -1,
            ..Default::default()
        });
    win.global::<BrowseApi>().set_hero(hero);
}

/// Focus moved to `idx`; provider info is requested once it stays put for `SETTLE`.
pub fn focus(a: &mut SlintApp, idx: i32) {
    a.browse.hero.focus = usize::try_from(idx).ok();
    push(a);
    if a.browse.hero.focus.is_some() {
        a.browse
            .hero
            .timer
            .start(TimerMode::SingleShot, SETTLE, || {
                with_app(settle);
            });
    }
}

fn settle(a: &mut SlintApp) {
    let row = a.browse.hero.focus.and_then(|i| a.streams.row(i));
    if let Some(row) = row.filter(|r| r.media != MediaKind::Live) {
        details::request(a, &(row.source_id, row.media, row.key));
    }
}

/// The row model was replaced: rebuild for the same index.
pub fn rows_reset(a: &mut SlintApp) {
    if let Some(i) = a.browse.hero.focus {
        focus(a, i as i32);
    }
}

pub fn connect(win: &AppWindow) {
    let api = win.global::<BrowseApi>();
    api.on_hero_focused(|i| {
        Timer::single_shot(Duration::ZERO, move || {
            with_app(|a| focus(a, i));
        });
    });
    api.on_hero_details(|i| {
        with_app(|a| super::row_action(a, i as usize, super::RowAction::Info));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resume_time_formats_from_duration_and_fraction() {
        assert_eq!(parse_secs("02:00:00"), Some(7200));
        assert_eq!(parse_secs("45:10"), Some(2710));
        assert_eq!(parse_secs("108 min"), None);
        assert_eq!(fmt_hms((7200.0 * 0.6) as u64), "1:12:00");
    }

    #[test]
    fn meta_joins_year_duration_rating() {
        let i = MediaInfo {
            release_date: Some("1999-03-31".into()),
            duration: Some("02:16:00".into()),
            rating: Some("8.7".into()),
            ..Default::default()
        };
        assert_eq!(meta(&i), "1999 · 02:16:00 · 8.7");
        assert_eq!(meta(&MediaInfo::default()), "");
    }
}
