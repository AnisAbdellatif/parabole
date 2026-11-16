//! Pushes the area's state into `SportsApi`.
use super::{enabled, filter, local_tz, now, rows};
use crate::slint_ui::state::SlintApp;
use crate::slint_ui::{SportsApi, SportsChannel, SportsRow};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};

pub(super) fn push(a: &SlintApp) {
    let Some(w) = a.win.upgrade() else { return };
    let f = &a.sports;
    let (tz, now) = (local_tz(), now());
    let langs = &f.settings.langs;
    let shown = f.shown();
    let q = f.query();
    let rows: Vec<SportsRow> = rows::rows(&shown, langs, q.searching(), now, tz)
        .into_iter()
        .map(|r| SportsRow {
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
    let channels: Vec<SportsChannel> = channels
        .into_iter()
        .map(|c| SportsChannel {
            header: c.header,
            text: c.text.into(),
            channel: c.channel.into(),
        })
        .collect();
    let api = w.global::<SportsApi>();
    let on = enabled(&f.settings);
    let cur = f.current();
    let names: Vec<SharedString> = on.iter().map(|s| s.name.into()).collect();
    api.set_sport(on.iter().position(|s| s.id == cur).map_or(0, |i| i as i32));
    api.set_sport_names(ModelRc::new(VecModel::from(names)));
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
        "No live events right now.".to_string()
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
