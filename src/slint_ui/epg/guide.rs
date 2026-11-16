//! Guide panel for one channel.
use super::format::{fraction, hm, now_secs};
use super::guide_pick;
use crate::model::{GuideId, SourceId, StreamRow};
use crate::slint_ui::state::{SlintApp, with_app};
use crate::slint_ui::{AppWindow, BrowseApi, GuideItem};
use slint::{ComponentHandle, ModelRc, VecModel};

const BEFORE: i64 = 2 * 3600;
const AFTER: i64 = 24 * 3600;

/// The stream whose guide is open; `hits` are the channel search results while `picking`.
pub struct Open {
    pub id: i64,
    pub name: String,
    pub source: SourceId,
    pub key: String,
    pub picking: bool,
    pub hits: Vec<(GuideId, String)>,
}

pub fn open(a: &mut SlintApp, row: &StreamRow) {
    a.epg.guide = Some(Open {
        id: row.id,
        name: row.name.clone(),
        source: row.source_id,
        key: row.key.clone(),
        picking: false,
        hits: Vec::new(),
    });
    push(a);
}

pub fn close(a: &mut SlintApp) {
    a.epg.guide = None;
    push(a);
}

/// Mirrors the open guide into `BrowseApi`; no-op while closed.
pub fn push(a: &SlintApp) {
    let Some(win) = a.win.upgrade() else {
        return;
    };
    let api = win.global::<BrowseApi>();
    let Some(g) = &a.epg.guide else {
        api.set_guide_open(false);
        return;
    };
    let (id, name) = (&g.id, &g.name);
    let now = now_secs();
    let items: Vec<GuideItem> =
        a.db.epg_schedule(*id, now - BEFORE, now + AFTER)
            .unwrap_or_default()
            .into_iter()
            .map(|p| {
                let airing = p.start <= now && now < p.stop;
                GuideItem {
                    time: format!("{}–{}", hm(p.start), hm(p.stop)).into(),
                    progress: if airing { fraction(&p, now) } else { 0.0 },
                    now: airing,
                    descr: p.descr.unwrap_or_default().into(),
                    title: p.title.into(),
                }
            })
            .collect();
    let first = items.iter().position(|i| i.now).unwrap_or(0);
    api.set_guide_title(name.as_str().into());
    api.set_guide(ModelRc::new(VecModel::from(items)));
    api.set_guide_scroll_to(first as i32);
    api.set_guide_picking(g.picking);
    api.set_guide_has_link(a.db.manual_link(g.source, &g.key).ok().flatten().is_some());
    api.set_guide_open(true);
}

pub fn connect(win: &AppWindow) {
    win.global::<BrowseApi>().on_guide_closed(|| {
        with_app(close);
    });
    guide_pick::connect(win);
}
