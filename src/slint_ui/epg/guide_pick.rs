//! Picking a guide channel by hand for the open guide's stream.
use super::guide::{self, Open};
use crate::slint_ui::state::{SlintApp, with_app};
use crate::slint_ui::{AppWindow, BrowseApi};
use crate::worker::Command;
use slint::{ComponentHandle, ModelRc, VecModel};

const MIN_CHARS: usize = 2;
const LIMIT: usize = 50;

fn set_hits(a: &mut SlintApp, labels: Vec<slint::SharedString>) {
    if let Some(w) = a.win.upgrade() {
        w.global::<BrowseApi>()
            .set_guide_hits(ModelRc::new(VecModel::from(labels)));
    }
}

fn start(a: &mut SlintApp) {
    if let Some(g) = a.epg.guide.as_mut() {
        g.picking = true;
        g.hits.clear();
    }
    if let Some(w) = a.win.upgrade() {
        w.global::<BrowseApi>().set_guide_query("".into());
    }
    set_hits(a, Vec::new());
    guide::push(a);
}

fn cancel(a: &mut SlintApp) {
    if let Some(g) = a.epg.guide.as_mut() {
        g.picking = false;
    }
    guide::push(a);
}

fn query(a: &mut SlintApp, text: &str) {
    let text = text.trim();
    let found = if text.chars().count() < MIN_CHARS {
        Ok(Vec::new())
    } else {
        a.db.search_guide_channels(text, LIMIT)
    };
    match found {
        Ok(rows) => {
            let labels = rows
                .iter()
                .map(|(_, guide, ch, name)| format!("{name} — {guide} ({ch})").into())
                .collect();
            if let Some(g) = a.epg.guide.as_mut() {
                g.hits = rows.into_iter().map(|(id, _, ch, _)| (id, ch)).collect();
            }
            set_hits(a, labels);
        }
        Err(e) => a.toast(format!("Guide search failed: {e:#}")),
    }
}

fn pick(a: &mut SlintApp, i: usize) {
    let Some(Open {
        source, key, hits, ..
    }) = &a.epg.guide
    else {
        return;
    };
    let Some((guide_id, channel)) = hits.get(i).cloned() else {
        return;
    };
    let (source, key) = (*source, key.clone());
    a.write_with(
        move |db| db.set_manual_link(source, &key, guide_id, &channel),
        move |a, ok| {
            if ok {
                a.worker.send(Command::RefreshEpg(guide_id));
            }
        },
    );
    cancel(a);
}

fn automatic(a: &mut SlintApp) {
    let Some(g) = &a.epg.guide else { return };
    let (source, key) = (g.source, g.key.clone());
    a.write_with(
        move |db| db.clear_manual_link(source, &key),
        |a, ok| {
            if ok {
                a.worker.send(Command::RelinkEpg);
            }
            guide::push(a);
        },
    );
}

pub fn connect(win: &AppWindow) {
    let api = win.global::<BrowseApi>();
    api.on_guide_pick_start(|| {
        with_app(start);
    });
    api.on_guide_pick_cancel(|| {
        with_app(cancel);
    });
    api.on_guide_auto(|| {
        with_app(automatic);
    });
    api.on_guide_query_edited(|t| {
        with_app(|a| query(a, &t));
    });
    api.on_guide_hit_picked(|i| {
        with_app(|a| pick(a, usize::try_from(i).unwrap_or(usize::MAX)));
    });
}
