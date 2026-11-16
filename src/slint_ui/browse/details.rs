//! Movie/series details window and provider-info cache (ports `ui/details_view.rs`).
use super::meta_fields;
use crate::model::{MediaInfo, MediaKind, Meta, SourceId, SourceKind, StreamRow};
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
    /// Online details per key; `None` once looked up without a match.
    pub meta: HashMap<Key, Option<Meta>>,
    meta_loading: HashSet<Key>,
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

/// Sends `LoadMeta` once per movie/series key, for every source kind.
pub fn request_meta(a: &mut SlintApp, key: &Key) {
    let wanted = matches!(key.1, MediaKind::Movie | MediaKind::Series);
    let st = &mut a.browse.info;
    if wanted && !st.meta.contains_key(key) && st.meta_loading.insert(key.clone()) {
        a.worker.send(Command::LoadMeta {
            source: key.0,
            media: key.1,
            key: key.2.clone(),
        });
    }
}

pub fn meta_loaded(
    a: &mut SlintApp,
    source: SourceId,
    media: MediaKind,
    key: String,
    meta: Option<Meta>,
) {
    let key = (source, media, key);
    a.browse.info.meta_loading.remove(&key);
    a.browse.info.meta.insert(key, meta);
    push(a);
    super::series::push(a);
}

pub fn open(a: &mut SlintApp, row: &StreamRow) {
    let key = (row.source_id, row.media, row.key.clone());
    a.browse.info.open = Some(Open {
        key: key.clone(),
        title: row.name.clone(),
        logo: row.logo.clone(),
    });
    request(a, &key);
    request_meta(a, &key);
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
        || a.browse
            .info
            .meta
            .values()
            .flatten()
            .any(|m| m.poster.as_deref() == Some(url))
        || a.browse.info.open.as_ref().and_then(|o| o.logo.as_deref()) == Some(url);
    if shown {
        push(a);
        super::series::push(a);
    }
}

/// `fallback_poster` is used when neither the online details nor the provider have a cover.
pub fn view(a: &SlintApp, key: &Key, fallback_poster: Option<&str>) -> InfoView {
    let st = &a.browse.info;
    let provider = st.cache.get(key).filter(|_| !st.failed.contains(key));
    let meta = st.meta.get(key).and_then(Option::as_ref);
    let in_flight = st.loading.contains(key) || st.meta_loading.contains(key);
    let state = if provider.is_some() || meta.is_some() {
        2
    } else if in_flight {
        1
    } else if st.failed.contains(key) {
        3
    } else {
        0
    };
    let fields: Vec<FieldItem> = meta_fields::fields(provider, meta, key.1)
        .into_iter()
        .map(|(l, v)| FieldItem {
            label: l.into(),
            value: v.into(),
        })
        .collect();
    let url = meta
        .and_then(|m| m.poster.as_deref())
        .or(provider.and_then(|i| i.cover.as_deref()))
        .or(fallback_poster);
    let poster = url.and_then(|u| a.images.get(u, Size::Poster));
    InfoView {
        state,
        has_poster: poster.is_some(),
        poster: poster.unwrap_or_default(),
        fields: ModelRc::new(VecModel::from(fields)),
    }
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
