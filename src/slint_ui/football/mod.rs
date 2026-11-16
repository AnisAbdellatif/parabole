//! Football tab: asks the worker for matches, shows them, refreshes while live.
mod filter;
mod rows;

use super::state::{SlintApp, with_app};
use super::{AppWindow, BrowseApi, Core, FootballApi, FootballChannel, FootballRow};
use crate::model::{Match, MatchStatus, ProviderStatus};
use crate::worker::football_settings::FootballSettings;
use crate::worker::{Command, Event};
use chrono::FixedOffset;
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use std::time::Duration;

const TICK: Duration = Duration::from_secs(60);
const SOON_S: i64 = 60;
const LATE_S: i64 = 3 * 3600;

pub struct FootballArea {
    pub open: bool,
    pub settings: FootballSettings,
    matches: Vec<Match>,
    providers: Vec<ProviderStatus>,
    loading: bool,
    updated_at: Option<i64>,
    selected: Option<String>,
    search: String,
    day: i64,
    status: filter::Status,
    timer: slint::Timer,
}

impl FootballArea {
    fn query(&self) -> filter::Query<'_> {
        filter::Query {
            search: &self.search,
            day: self.day,
            status: self.status,
        }
    }

    fn shown(&self) -> Vec<&Match> {
        filter::visible(&self.matches, &self.query(), now(), local_tz())
    }
}

impl FootballArea {
    pub fn new(settings: FootballSettings) -> Self {
        Self {
            open: false,
            settings,
            matches: Vec::new(),
            providers: Vec::new(),
            loading: false,
            updated_at: None,
            selected: None,
            search: String::new(),
            day: 0,
            status: filter::Status::All,
            timer: Default::default(),
        }
    }
}

// shortcut: one UTC offset for the whole week, wrong by 1 h across a DST change; use Local per timestamp if that matters
fn local_tz() -> FixedOffset {
    *chrono::Local::now().offset()
}

fn now() -> i64 {
    chrono::Utc::now().timestamp()
}

/// Any live match, or one about to start or that should have started recently.
fn wants_tick(matches: &[Match], now: i64) -> bool {
    matches.iter().any(|m| match m.status {
        MatchStatus::Live { .. } => true,
        MatchStatus::Upcoming => m.kickoff_utc <= now + SOON_S && m.kickoff_utc >= now - LATE_S,
        MatchStatus::Finished => false,
    })
}

pub fn connect(win: &AppWindow) {
    let api = win.global::<FootballApi>();
    api.on_row_selected(|i| {
        with_app(|a| {
            let f = &mut a.football;
            f.selected = usize::try_from(i)
                .ok()
                .and_then(|i| f.shown().get(i).map(|m| m.key.clone()));
            push(a);
        });
    });
    api.on_search_edited(|text| {
        with_app(|a| {
            a.football.search = text.to_string();
            push(a);
        });
    });
    api.on_filter_picked(|i| {
        with_app(|a| {
            a.football.status = filter::Status::from_index(i);
            push(a);
        });
    });
    api.on_day_step(|d| {
        with_app(|a| {
            let f = &mut a.football;
            f.day = (f.day + i64::from(d)).clamp(0, filter::LAST_DAY);
            push(a);
        });
    });
    api.on_channel_clicked(|i| {
        with_app(|a| {
            let Some(w) = a.win.upgrade() else { return };
            let ch = usize::try_from(i)
                .ok()
                .and_then(|i| w.global::<FootballApi>().get_channels().row_data(i));
            if let Some(ch) = ch.filter(|c| !c.header) {
                super::browse::copy_text(a, &ch.channel, "Channel name copied");
            }
        });
    });
    api.on_refresh(|| {
        with_app(request);
    });
}

pub fn open(a: &mut SlintApp) {
    if !a.football.settings.enabled {
        return;
    }
    a.football.open = true;
    if let Some(w) = a.win.upgrade() {
        w.global::<Core>().set_tab(3);
        w.global::<BrowseApi>().set_sidebar_focus(false);
    }
    push(a);
    request(a);
    a.football
        .timer
        .start(slint::TimerMode::Repeated, TICK, || {
            with_app(|a| {
                if a.football.open && wants_tick(&a.football.matches, now()) {
                    request(a);
                }
            });
        });
}

pub fn leave(a: &mut SlintApp) {
    a.football.open = false;
    a.football.timer.stop();
}

pub fn apply_settings(a: &mut SlintApp, s: FootballSettings) {
    let enabled = s.enabled;
    a.football.settings = s;
    if let Some(w) = a.win.upgrade() {
        w.global::<Core>().set_football_enabled(enabled);
    }
    if !a.football.open {
        return;
    }
    if enabled {
        request(a);
    } else {
        super::callbacks::switch_tab(a, a.state.tab as i32);
    }
}

pub fn on_event(a: &mut SlintApp, ev: Event) -> Option<Event> {
    let Event::Football(u) = ev else {
        return Some(ev);
    };
    let f = &mut a.football;
    f.matches = u.matches;
    if !u.providers.is_empty() {
        f.providers = u.providers;
    }
    if u.done {
        f.loading = false;
        f.updated_at = Some(now());
    }
    if let Some(k) = &f.selected
        && !f.matches.iter().any(|m| &m.key == k)
    {
        f.selected = None;
    }
    push(a);
    None
}

fn request(a: &mut SlintApp) {
    a.football.loading = true;
    a.worker.send(Command::Football {
        langs: a.football.settings.langs.clone(),
    });
    push(a);
}

fn push(a: &SlintApp) {
    let Some(w) = a.win.upgrade() else { return };
    let f = &a.football;
    let (tz, now) = (local_tz(), now());
    let langs = &f.settings.langs;
    let shown = f.shown();
    let q = f.query();
    let rows: Vec<FootballRow> = rows::rows(&shown, langs, q.searching(), now, tz)
        .into_iter()
        .map(|r| FootballRow {
            key: r.key.into(),
            section: r.section.into(),
            time: r.time.into(),
            live: r.live,
            home: r.home.into(),
            away: r.away.into(),
            home_score: r.home_score.into(),
            away_score: r.away_score.into(),
            langs: r.langs.into(),
        })
        .collect();
    let pos = f
        .selected
        .as_ref()
        .and_then(|k| shown.iter().position(|m| &m.key == k));
    let (title, sub, channels) =
        pos.map_or_else(Default::default, |i| rows::details(shown[i], langs, tz));
    let channels: Vec<FootballChannel> = channels
        .into_iter()
        .map(|c| FootballChannel {
            header: c.header,
            text: c.text.into(),
            channel: c.channel.into(),
        })
        .collect();
    let api = w.global::<FootballApi>();
    api.set_rows(ModelRc::new(VecModel::from(rows)));
    api.set_cursor(pos.map_or(-1, |i| i as i32));
    api.set_detail_title(title.into());
    api.set_detail_sub(sub.into());
    api.set_channels(ModelRc::new(VecModel::from(channels)));
    api.set_status(rows::status(&f.providers, f.loading, f.updated_at, tz).into());
    let empty = if f.matches.is_empty() {
        rows::empty_text(f.loading).to_string()
    } else if q.searching() {
        format!("No match for “{}”.", f.search.trim())
    } else if f.status == filter::Status::Live {
        "No live matches right now.".to_string()
    } else {
        rows::empty_text(false).to_string()
    };
    api.set_filter(f.status.index());
    api.set_live_count(f.matches.iter().filter(|m| filter::is_live(m)).count() as i32);
    api.set_day_label(rows::day_label(f.day, now, tz).into());
    api.set_can_prev(f.day > 0);
    api.set_can_next(f.day < filter::LAST_DAY);
    api.set_day_enabled(!q.searching() && f.status != filter::Status::Live);
    api.set_empty_text(empty.into());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m(status: MatchStatus, kickoff_utc: i64) -> Match {
        Match {
            key: "k".into(),
            competition: "c".into(),
            home: "h".into(),
            away: "a".into(),
            kickoff_utc,
            status,
            broadcasts: vec![],
        }
    }

    #[test]
    fn tick_only_when_something_is_happening() {
        let now = 100_000;
        assert!(!wants_tick(&[], now));
        assert!(!wants_tick(&[m(MatchStatus::Upcoming, now + 3600)], now));
        assert!(wants_tick(&[m(MatchStatus::Upcoming, now + 30)], now));
        assert!(wants_tick(&[m(MatchStatus::Upcoming, now - 3600)], now));
        assert!(!wants_tick(
            &[m(MatchStatus::Upcoming, now - 4 * 3600)],
            now
        ));
        assert!(!wants_tick(&[m(MatchStatus::Finished, now - 60)], now));
        let live = MatchStatus::Live {
            clock: "1'".into(),
            home: 0,
            away: 0,
        };
        assert!(wants_tick(&[m(live, now - 9999)], now));
    }
}
