//! Cmd/Ctrl+K command palette: search results arrive from the worker, the rest is built locally.
mod entries;
mod fuzzy;
mod run;

use super::state::{SlintApp, with_app};
use super::{AppWindow, Core, PaletteApi, PaletteItem};
use crate::model::{MediaKind, StreamRow};
use crate::worker::{Command, Event};
use entries::{Entry, actions, media_label};
use slint::{ComponentHandle, ModelRc, VecModel};
use std::time::Duration;

const DEBOUNCE: Duration = Duration::from_millis(100);
const MAX_CATEGORIES: usize = 8;

#[derive(Default)]
pub struct PaletteArea {
    query: String,
    seq: u64,
    streams: Vec<StreamRow>,
    entries: Vec<Entry>,
    timer: slint::Timer,
}

pub fn connect(win: &AppWindow) {
    win.global::<Core>().on_open_palette(|| {
        with_app(|a| a.open_palette());
    });
    let api = win.global::<PaletteApi>();
    api.on_edited(|q| {
        with_app(|a| a.palette_edited(q.as_str()));
    });
    api.on_activate(|i| {
        with_app(|a| a.palette_activate(i));
    });
}

/// Consumes `SearchResults`; everything else goes on.
pub fn on_event(a: &mut SlintApp, ev: Event) -> Option<Event> {
    match ev {
        Event::SearchResults { seq, rows } => {
            if seq == a.palette.seq {
                a.palette.streams = rows;
                a.palette_rebuild();
            }
            None
        }
        ev => Some(ev),
    }
}

impl SlintApp {
    fn open_palette(&mut self) {
        self.palette.query.clear();
        self.palette.streams.clear();
        self.palette.seq += 1;
        self.palette.timer.stop();
        self.palette_rebuild();
        if let Some(w) = self.win.upgrade() {
            let api = w.global::<PaletteApi>();
            api.set_query("".into());
            api.set_open(true);
        }
    }

    fn palette_edited(&mut self, q: &str) {
        self.palette.query = q.trim().to_owned();
        self.palette.seq += 1;
        self.palette.streams.clear();
        self.palette_rebuild();
        if self.palette.query.is_empty() {
            self.palette.timer.stop();
            return;
        }
        self.palette
            .timer
            .start(slint::TimerMode::SingleShot, DEBOUNCE, || {
                with_app(|a| {
                    let (profile, query, seq) =
                        (a.profile(), a.palette.query.clone(), a.palette.seq);
                    a.worker.send(Command::Search {
                        profile,
                        query,
                        seq,
                    });
                });
            });
    }

    /// Streams (DB order), then fuzzy-ranked categories, then fuzzy-ranked actions.
    fn palette_rebuild(&mut self) {
        let q = self.palette.query.clone();
        let mut entries: Vec<Entry> = self
            .palette
            .streams
            .iter()
            .cloned()
            .map(Entry::Stream)
            .collect();
        let multi = self.sources.len() > 1;
        let mut items: Vec<PaletteItem> = self
            .palette
            .streams
            .iter()
            .map(|r| {
                let src = self.sources.iter().find(|s| s.id == r.source_id);
                let src = src.filter(|_| multi).map(|s| s.name.as_str());
                item(kind_of(r.media), &r.name, &stream_subtitle(r.media, src))
            })
            .collect();
        if !q.is_empty() {
            let mut cats = Vec::new();
            let tabs = [self.state.tab]
                .into_iter()
                .chain(MediaKind::ALL.into_iter().filter(|m| *m != self.state.tab));
            for m in tabs {
                let rows = self
                    .db
                    .categories(self.profile(), None, m, false)
                    .unwrap_or_default();
                cats.extend(
                    rows.into_iter()
                        .filter_map(|c| fuzzy::score(&q, &c.name).map(|s| (s, m, c))),
                );
            }
            // stable: equal scores keep current-tab-first order
            cats.sort_by_key(|(s, ..)| std::cmp::Reverse(*s));
            for (_, m, c) in cats.into_iter().take(MAX_CATEGORIES) {
                items.push(item(3, &c.name, &format!("{} category", media_label(m))));
                entries.push(Entry::Category(m, c.source_id, c.key));
            }
        }
        let mut acts: Vec<_> = actions(&self.profiles)
            .into_iter()
            .filter_map(|(t, act)| fuzzy::score(&q, &t).map(|s| (s, t, act)))
            .collect();
        if !q.is_empty() {
            acts.sort_by_key(|(s, ..)| std::cmp::Reverse(*s));
        }
        for (_, t, act) in acts {
            items.push(item(4, &t, "Action"));
            entries.push(Entry::Action(act));
        }
        self.palette.entries = entries;
        if let Some(w) = self.win.upgrade() {
            let api = w.global::<PaletteApi>();
            api.set_items(ModelRc::new(VecModel::from(items)));
            api.set_cursor(0);
        }
    }

    fn palette_activate(&mut self, i: i32) {
        let Some(entry) = usize::try_from(i)
            .ok()
            .and_then(|i| self.palette.entries.get(i))
        else {
            return;
        };
        let entry = entry.clone();
        if let Some(w) = self.win.upgrade() {
            w.global::<PaletteApi>().set_open(false);
        }
        self.run_entry(entry);
    }
}

fn kind_of(m: MediaKind) -> i32 {
    match m {
        MediaKind::Live => 0,
        MediaKind::Movie => 1,
        MediaKind::Series => 2,
    }
}

// shortcut: kind instead of category name (StreamRow has no category), add a batched category lookup in db/search.rs to show it.
/// Stream row subtitle: kind, plus " · source" when several sources exist.
fn stream_subtitle(m: MediaKind, source: Option<&str>) -> String {
    match source {
        Some(s) => format!("{} · {s}", media_label(m)),
        None => media_label(m).to_owned(),
    }
}

fn item(kind: i32, title: &str, subtitle: &str) -> PaletteItem {
    PaletteItem {
        kind,
        title: title.into(),
        subtitle: subtitle.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subtitle_adds_source_only_when_given() {
        assert_eq!(stream_subtitle(MediaKind::Live, None), "Live");
        assert_eq!(
            stream_subtitle(MediaKind::Movie, Some("Home")),
            "Movie · Home"
        );
    }
}
