//! Property observation and event draining for the embedded mpv.

use crate::player::state::{Notice, PlayerState, parse_tracks};
use libmpv2::events::{Event, PropertyData};
use libmpv2::{Format, Mpv, mpv_end_file_reason};

const OBSERVED: [(&str, Format); 8] = [
    ("pause", Format::Flag),
    ("time-pos", Format::Double),
    ("duration", Format::Double),
    ("volume", Format::Double),
    ("mute", Format::Flag),
    ("paused-for-cache", Format::Flag),
    ("media-title", Format::String),
    ("track-list/count", Format::Int64),
];

pub fn observe_all(mpv: &Mpv) -> libmpv2::Result<()> {
    for (i, (name, format)) in OBSERVED.into_iter().enumerate() {
        mpv.observe_property(name, format, i as u64 + 1)?;
    }
    Ok(())
}

fn apply(state: &mut PlayerState, mpv: &Mpv, name: &str, change: &PropertyData<'_>) {
    use PropertyData::*;
    match (name, change) {
        ("pause", Flag(v)) => state.paused = *v,
        ("time-pos", Double(v)) => state.time_pos = Some(*v),
        ("time-pos", None) => state.time_pos = Option::None,
        ("duration", Double(v)) => state.duration = Some(*v),
        ("duration", None) => state.duration = Option::None,
        ("volume", Double(v)) => state.volume = *v,
        ("mute", Flag(v)) => state.muted = *v,
        ("paused-for-cache", Flag(v)) => state.buffering = *v,
        ("media-title", Str(v)) => state.media_title = Some(v.to_string()),
        ("track-list/count", _) => {
            // the track list is a node property; its string form is JSON
            let json = mpv.get_property::<String>("track-list").unwrap_or_default();
            state.tracks = parse_tracks(&json);
        }
        _ => {}
    }
}

/// Drains the event queue into `state`; returns what needs the owner's attention.
pub fn poll(mpv: &Mpv, state: &mut PlayerState) -> Vec<Notice> {
    let mut notices = Vec::new();
    while let Some(ev) = mpv.wait_event(0.0) {
        match ev {
            Ok(Event::PropertyChange { name, change, .. }) => apply(state, mpv, name, &change),
            Ok(Event::StartFile) => state.reset_file(),
            Ok(Event::FileLoaded) => notices.push(Notice::Loaded),
            Ok(Event::EndFile(reason)) if reason == mpv_end_file_reason::Eof => {
                notices.push(Notice::Ended { failed: false })
            }
            // libmpv2 reports END_FILE with an error code as Err
            Err(_) => notices.push(Notice::Ended { failed: true }),
            _ => {}
        }
    }
    notices
}
