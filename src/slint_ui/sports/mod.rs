//! Sports tab: asks the worker for matches, shows them, refreshes while live.
mod filter;
mod push;
mod rows;

use super::state::{SlintApp, with_app};
use super::{AppWindow, BrowseApi, Core, SportsApi};
use crate::model::{Match, MatchStatus, ProviderStatus};
use crate::sports::registry::{SPORTS, Sport};
use crate::worker::sports_settings::SportsSettings;
use crate::worker::{Command, Event};
use push::push;
use slint::{ComponentHandle, Model};
use std::time::Duration;

const TICK: Duration = Duration::from_secs(60);
const SOON_S: i64 = 60;
const LATE_S: i64 = 3 * 3600;

pub struct SportsArea {
    pub open: bool,
    pub settings: SportsSettings,
    sport: String,
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

impl SportsArea {
    fn query(&self) -> filter::Query<'_> {
        filter::Query {
            search: &self.search,
            day: self.day,
            status: self.status,
        }
    }

    /// The stored sport if still enabled, else the first enabled one.
    fn current(&self) -> String {
        let on = enabled(&self.settings);
        on.iter()
            .find(|s| s.id == self.sport)
            .or(on.first())
            .map_or_else(|| self.sport.clone(), |s| s.id.to_string())
    }

    /// Makes `id` the current sport and drops what belonged to the old one; filter and day stay.
    fn switch_to(&mut self, id: String) {
        self.sport = id;
        self.matches.clear();
        self.providers.clear();
        self.selected = None;
        self.updated_at = None;
        self.search.clear();
    }

    fn shown(&self) -> Vec<&Match> {
        filter::visible(&self.matches, &self.query(), now(), &local_tz())
    }
}

impl SportsArea {
    pub fn new(settings: SportsSettings) -> Self {
        Self {
            open: false,
            settings,
            sport: "football".into(),
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

/// Registry sports the user turned on, in registry order.
fn enabled(settings: &SportsSettings) -> Vec<&'static Sport> {
    SPORTS
        .iter()
        .copied()
        .filter(|s| settings.sports.iter().any(|id| id == s.id))
        .collect()
}

fn local_tz() -> chrono::Local {
    chrono::Local
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
    let api = win.global::<SportsApi>();
    api.on_row_selected(|i| {
        with_app(|a| {
            let f = &mut a.sports;
            f.selected = usize::try_from(i)
                .ok()
                .and_then(|i| f.shown().get(i).map(|m| m.key.clone()));
            push(a);
        });
    });
    api.on_sport_picked(|i| {
        with_app(|a| {
            let on = enabled(&a.sports.settings);
            let Some(s) = usize::try_from(i).ok().and_then(|i| on.get(i)) else {
                return;
            };
            a.sports.switch_to(s.id.to_string());
            if let Some(w) = a.win.upgrade() {
                w.global::<SportsApi>().set_search("".into());
            }
            request(a);
        });
    });
    api.on_search_edited(|text| {
        with_app(|a| {
            a.sports.search = text.to_string();
            push(a);
        });
    });
    api.on_filter_picked(|i| {
        with_app(|a| {
            a.sports.status = filter::Status::from_index(i);
            push(a);
        });
    });
    api.on_day_step(|d| {
        with_app(|a| {
            let f = &mut a.sports;
            f.day = (f.day + i64::from(d)).clamp(0, filter::LAST_DAY);
            push(a);
        });
    });
    api.on_channel_clicked(|i| {
        with_app(|a| {
            let Some(w) = a.win.upgrade() else { return };
            let ch = usize::try_from(i)
                .ok()
                .and_then(|i| w.global::<SportsApi>().get_channels().row_data(i));
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
    if !a.sports.settings.enabled {
        return;
    }
    a.sports.open = true;
    if let Some(w) = a.win.upgrade() {
        w.global::<Core>().set_tab(3);
        w.global::<BrowseApi>().set_sidebar_focus(false);
    }
    push(a);
    request(a);
    a.sports.timer.start(slint::TimerMode::Repeated, TICK, || {
        with_app(|a| {
            if a.sports.open && wants_tick(&a.sports.matches, now()) {
                request(a);
            }
        });
    });
}

pub fn leave(a: &mut SlintApp) {
    a.sports.open = false;
    a.sports.timer.stop();
}

pub fn apply_settings(a: &mut SlintApp, s: SportsSettings) {
    let enabled = s.enabled;
    a.sports.settings = s;
    let cur = a.sports.current();
    if cur != a.sports.sport {
        a.sports.switch_to(cur);
        if let Some(w) = a.win.upgrade() {
            w.global::<SportsApi>().set_search("".into());
        }
    }
    if let Some(w) = a.win.upgrade() {
        w.global::<Core>().set_sports_enabled(enabled);
    }
    if !a.sports.open {
        return;
    }
    if enabled {
        request(a);
    } else {
        super::callbacks::switch_tab(a, a.state.tab as i32);
    }
}

pub fn on_event(a: &mut SlintApp, ev: Event) -> Option<Event> {
    let Event::Sports(u) = ev else {
        return Some(ev);
    };
    if u.sport != a.sports.current() {
        if u.done {
            request(a);
        }
        return None;
    }
    let f = &mut a.sports;
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
    a.sports.loading = true;
    a.worker.send(Command::Sports {
        sport: a.sports.current(),
        langs: a.sports.settings.langs.clone(),
    });
    push(a);
}

#[cfg(test)]
mod tests;
