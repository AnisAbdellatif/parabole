//! Guide panel for one channel.
use super::format::{fraction, hm, now_secs};
use crate::model::StreamRow;
use crate::slint_ui::state::{SlintApp, with_app};
use crate::slint_ui::{AppWindow, BrowseApi, GuideItem};
use slint::{ComponentHandle, ModelRc, VecModel};

const BEFORE: i64 = 2 * 3600;
const AFTER: i64 = 24 * 3600;

pub fn open(a: &mut SlintApp, row: &StreamRow) {
    a.epg.guide = Some((row.id, row.name.clone()));
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
    let Some((id, name)) = &a.epg.guide else {
        api.set_guide_open(false);
        return;
    };
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
    api.set_guide_open(true);
}

pub fn connect(win: &AppWindow) {
    win.global::<BrowseApi>().on_guide_closed(|| {
        with_app(close);
    });
}
