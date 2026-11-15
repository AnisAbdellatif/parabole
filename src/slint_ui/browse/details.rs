//! Movie/series details window and provider-info cache (ports `ui/details_view.rs`).
use crate::model::{MediaInfo, MediaKind, SourceId, SourceKind, StreamRow};
use crate::slint_ui::images::Size;
use crate::slint_ui::state::{SlintApp, with_app};
use crate::slint_ui::{AppWindow, BrowseApi, FieldItem, InfoView};
use crate::worker::Command;
use slint::{ComponentHandle, ModelRc, VecModel};
use std::collections::{HashMap, HashSet};

pub type Key = (SourceId, MediaKind, String);

pub struct Open {
    key: Key,
    title: String,
    logo: Option<String>,
}

/// Per-session cache of fetched details plus the currently open window.
#[derive(Default)]
pub struct InfoStore {
    cache: HashMap<Key, MediaInfo>,
    loading: HashSet<Key>,
    failed: HashSet<Key>,
    pub open: Option<Open>,
}

pub fn cached<'a>(a: &'a SlintApp, key: &Key) -> Option<&'a MediaInfo> {
    a.browse.info.cache.get(key)
}

/// Sends `LoadInfo` once per key; only Xtream sources have details.
pub fn request(a: &mut SlintApp, key: &Key) {
    let xtream = a
        .sources
        .iter()
        .any(|s| s.id == key.0 && s.kind == SourceKind::Xtream);
    if xtream && !a.browse.info.cache.contains_key(key) && a.browse.info.loading.insert(key.clone())
    {
        a.worker.send(Command::LoadInfo {
            source: key.0,
            media: key.1,
            key: key.2.clone(),
        });
    }
}

pub fn open(a: &mut SlintApp, row: &StreamRow) {
    let key = (row.source_id, row.media, row.key.clone());
    a.browse.info.open = Some(Open {
        key: key.clone(),
        title: row.name.clone(),
        logo: row.logo.clone(),
    });
    request(a, &key);
    push(a);
}

pub fn close(a: &mut SlintApp) -> bool {
    let was = a.browse.info.open.take().is_some();
    push(a);
    was
}

pub fn loaded(
    a: &mut SlintApp,
    source: SourceId,
    media: MediaKind,
    key: String,
    result: Result<MediaInfo, String>,
) {
    let key = (source, media, key);
    a.browse.info.loading.remove(&key);
    match result {
        Ok(info) => {
            a.browse.info.cache.insert(key, info);
        }
        Err(e) => {
            // cache an empty entry so the window stops waiting and no retry loop starts
            a.browse.info.failed.insert(key.clone());
            a.browse.info.cache.insert(key, MediaInfo::default());
            a.toast(format!("Could not load details: {e}"));
        }
    }
    push(a);
    super::series::push(a);
    super::hero::push(a);
}

/// A poster finished loading: redraw whichever info views show it.
pub fn poster_ready(a: &mut SlintApp, url: &str) {
    let shown = a
        .browse
        .info
        .cache
        .values()
        .any(|i| i.cover.as_deref() == Some(url))
        || a.browse.info.open.as_ref().and_then(|o| o.logo.as_deref()) == Some(url);
    if shown {
        push(a);
        super::series::push(a);
    }
}

fn fields(info: &MediaInfo) -> Vec<FieldItem> {
    [
        ("Genre", &info.genre),
        ("Released", &info.release_date),
        ("Duration", &info.duration),
        ("Rating", &info.rating),
        ("Director", &info.director),
        ("Cast", &info.cast),
        ("Plot", &info.plot),
    ]
    .into_iter()
    .filter_map(|(l, v)| {
        v.as_ref().map(|v| FieldItem {
            label: l.into(),
            value: v.as_str().into(),
        })
    })
    .collect()
}

/// `fallback_poster` is used when the provider sent no cover.
pub fn view(a: &SlintApp, key: &Key, fallback_poster: Option<&str>) -> InfoView {
    let st = &a.browse.info;
    let mut v = InfoView {
        state: 0,
        poster: Default::default(),
        has_poster: false,
        fields: ModelRc::default(),
    };
    match st.cache.get(key) {
        Some(_) if st.failed.contains(key) => v.state = 3,
        Some(info) => {
            v.state = 2;
            v.fields = ModelRc::new(VecModel::from(fields(info)));
            let url = info.cover.as_deref().or(fallback_poster);
            if let Some(img) = url.and_then(|u| a.images.get(u, Size::Poster)) {
                v.poster = img;
                v.has_poster = true;
            }
        }
        None if st.loading.contains(key) => v.state = 1,
        None => {
            if let Some(img) = fallback_poster.and_then(|u| a.images.get(u, Size::Poster)) {
                v.poster = img;
                v.has_poster = true;
            }
        }
    }
    v
}

pub fn push(a: &SlintApp) {
    let Some(win) = a.win.upgrade() else {
        return;
    };
    let api = win.global::<BrowseApi>();
    match &a.browse.info.open {
        Some(o) => {
            api.set_details_title(o.title.as_str().into());
            api.set_details(view(a, &o.key, o.logo.as_deref()));
            api.set_details_open(true);
        }
        None => api.set_details_open(false),
    }
}

pub fn connect(win: &AppWindow) {
    let api = win.global::<BrowseApi>();
    api.on_details_closed(|| {
        with_app(close);
    });
    api.on_details_retry(|| {
        with_app(|a| {
            let Some(key) = a.browse.info.open.as_ref().map(|o| o.key.clone()) else {
                return;
            };
            a.browse.info.cache.remove(&key);
            a.browse.info.failed.remove(&key);
            request(a, &key);
            push(a);
        });
    });
}
