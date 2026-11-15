//! Series page: seasons, episodes, watched marks (ports `ui/series_view.rs`).
use super::details;
use crate::db::progress::{Watch, classify};
use crate::model::{MediaKind, Season, SourceId, SourceKind, StreamRow};
use crate::slint_ui::state::{SlintApp, with_app};
use crate::slint_ui::{AppWindow, BrowseApi, EpisodeItem, MenuEntry};
use crate::worker::{Command, SeriesCtx, episode_key};
use slint::{ComponentHandle, ModelRc, VecModel};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

const PROGRESS_EVERY: Duration = Duration::from_secs(3);

pub struct SeriesState {
    source: SourceId,
    key: String,
    title: String,
    loading: bool,
    seasons: Vec<Season>,
    season: usize,
    /// Episode to start as soon as the seasons arrive (Continue watching).
    autoplay: Option<String>,
    /// Watched fraction by history key (`ep:<id>`), refreshed periodically.
    progress: HashMap<String, f32>,
}

fn season_name(se: &Season) -> String {
    if se.name.is_empty() {
        format!("Season {}", se.number)
    } else {
        se.name.clone()
    }
}

pub fn open(a: &mut SlintApp, row: &StreamRow) {
    let xtream = a
        .sources
        .iter()
        .any(|s| s.id == row.source_id && s.kind == SourceKind::Xtream);
    if !xtream {
        a.toast("Episode lists need an Xtream source");
        return;
    }
    a.browse.series = Some(SeriesState {
        source: row.source_id,
        key: row.key.clone(),
        title: row.name.clone(),
        loading: true,
        seasons: Vec::new(),
        season: 0,
        autoplay: row.resume_episode.as_ref().map(|e| e.id.clone()),
        progress: HashMap::new(),
    });
    a.worker.send(Command::LoadSeries {
        source: row.source_id,
        key: row.key.clone(),
    });
    details::request(a, &(row.source_id, MediaKind::Series, row.key.clone()));
    a.browse
        .series_timer
        .start(slint::TimerMode::Repeated, PROGRESS_EVERY, || {
            with_app(refresh_progress);
        });
    refresh_progress(a);
}

pub fn close(a: &mut SlintApp) {
    if a.browse.series.take().is_some() {
        a.browse.series_timer.stop();
        push(a);
    }
}

pub fn loaded(a: &mut SlintApp, source: SourceId, key: &str, result: Result<Vec<Season>, String>) {
    let Some(s) = a
        .browse
        .series
        .as_mut()
        .filter(|s| s.source == source && s.key == key)
    else {
        return;
    };
    s.loading = false;
    match result {
        Ok(seasons) => {
            s.seasons = seasons;
            if let Some(id) = s.autoplay.take() {
                play_episode(a, &id);
            }
            push(a);
        }
        Err(e) => {
            close(a);
            a.toast(format!("Could not load series: {e}"));
        }
    }
}

fn series_ctx(a: &SlintApp) -> Option<(SeriesCtx, SourceId)> {
    let s = a.browse.series.as_ref()?;
    let ctx = SeriesCtx {
        key: s.key.clone(),
        title: s.title.clone(),
        seasons: Arc::new(s.seasons.clone()),
    };
    Some((ctx, s.source))
}

fn play_episode(a: &mut SlintApp, episode_id: &str) {
    let Some((ctx, source)) = series_ctx(a) else {
        return;
    };
    if let Some(cmd) = ctx.play_command(a.profile(), source, episode_id) {
        a.worker.send(cmd);
    }
}

fn toggle_watched(a: &mut SlintApp, episode_id: &str, watched: bool) {
    let Some((ctx, source)) = series_ctx(a) else {
        return;
    };
    let profile = a.profile();
    let key = episode_key(episode_id);
    let res = if watched {
        let label = crate::episodes::find(&ctx.seasons, episode_id)
            .map_or_else(String::new, |(s, e)| crate::episodes::label(s, e));
        a.db.record_episode_watch(profile, source, &key, &label, &ctx.key)
            .and_then(|_| {
                a.db.set_watched(profile, source, MediaKind::Series, &key, true)
            })
    } else {
        a.db.set_watched(profile, source, MediaKind::Series, &key, false)
    };
    a.report(res);
    refresh_progress(a);
}

fn refresh_progress(a: &mut SlintApp) {
    let profile = a.profile();
    let Some(source) = a.browse.series.as_ref().map(|s| s.source) else {
        return;
    };
    if let Ok(map) = a.db.episode_progress(profile, source)
        && let Some(s) = a.browse.series.as_mut()
        && s.progress != map
    {
        s.progress = map;
        push(a);
    }
}

pub fn push(a: &SlintApp) {
    let Some(win) = a.win.upgrade() else {
        return;
    };
    let api = win.global::<BrowseApi>();
    let Some(s) = &a.browse.series else {
        api.set_series_open(false);
        a.browse.episodes.set_vec(Vec::new());
        return;
    };
    api.set_series_open(true);
    api.set_series_title(s.title.as_str().into());
    api.set_series_loading(s.loading);
    api.set_series_empty(!s.loading && s.seasons.is_empty());
    let key = (s.source, MediaKind::Series, s.key.clone());
    let mut info = details::view(a, &key, None);
    if info.state != 2 {
        info.state = 0;
    }
    api.set_series_info(info);
    let Some(season) = s.seasons.get(s.season) else {
        a.browse.episodes.set_vec(Vec::new());
        return;
    };
    api.set_season_label(season_name(season).into());
    let items: Vec<MenuEntry> = s
        .seasons
        .iter()
        .enumerate()
        .map(|(i, se)| MenuEntry {
            id: format!("season:{i}").into(),
            label: format!(
                "{}{}",
                if i == s.season { "✓ " } else { "    " },
                season_name(se)
            )
            .into(),
            kind: 0,
        })
        .collect();
    api.set_season_items(ModelRc::new(VecModel::from(items)));
    let eps = season.episodes.iter().map(|ep| {
        let p = s.progress.get(&episode_key(&ep.id)).copied();
        EpisodeItem {
            id: ep.id.as_str().into(),
            num: ep
                .episode_num
                .map(|n| format!("E{n}"))
                .unwrap_or_default()
                .into(),
            title: ep.title.as_str().into(),
            progress: if classify(p) == Watch::Partial {
                p.unwrap_or(0.0)
            } else {
                0.0
            },
            watched: classify(p) == Watch::Watched,
        }
    });
    a.browse.episodes.set_vec(eps.collect::<Vec<_>>());
}

fn episode_id(a: &SlintApp, i: i32) -> Option<(String, bool)> {
    let s = a.browse.series.as_ref()?;
    let ep = s
        .seasons
        .get(s.season)?
        .episodes
        .get(usize::try_from(i).ok()?)?;
    let watched = classify(s.progress.get(&episode_key(&ep.id)).copied()) == Watch::Watched;
    Some((ep.id.clone(), watched))
}

pub fn connect(win: &AppWindow) {
    let api = win.global::<BrowseApi>();
    api.on_series_back(|| {
        with_app(close);
    });
    api.on_season_picked(|id| {
        with_app(|a| {
            let i = id.strip_prefix("season:").and_then(|n| n.parse().ok());
            if let (Some(i), Some(s)) = (i, a.browse.series.as_mut()) {
                s.season = i;
                push(a);
            }
        });
    });
    api.on_episode_activated(|i| {
        with_app(|a| {
            if let Some((id, _)) = episode_id(a, i) {
                play_episode(a, &id);
            }
        });
    });
    api.on_episode_watched_toggled(|i| {
        with_app(|a| {
            if let Some((id, watched)) = episode_id(a, i) {
                toggle_watched(a, &id, !watched);
            }
        });
    });
}
