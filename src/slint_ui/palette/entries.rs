//! What the palette can list and run.
use crate::model::{MediaKind, Profile, SourceId, StreamRow};

#[derive(Clone, Debug)]
pub enum Action {
    RefreshAll,
    ThemeMode,
    Density,
    ReduceMotion,
    ToggleDock,
    OpenDialog(&'static str),
    SwitchProfile(usize),
}

#[derive(Clone, Debug)]
pub enum Entry {
    Stream(StreamRow),
    Category(MediaKind, SourceId, String),
    Action(Action),
}

pub fn media_label(m: MediaKind) -> &'static str {
    match m {
        MediaKind::Live => "Live",
        MediaKind::Movie => "Movie",
        MediaKind::Series => "Series",
    }
}

/// Every action with its title; one "Switch to profile" per profile.
pub fn actions(profiles: &[Profile]) -> Vec<(String, Action)> {
    let mut v: Vec<(String, Action)> = vec![
        ("Refresh all sources".into(), Action::RefreshAll),
        ("Switch theme mode".into(), Action::ThemeMode),
        ("Cycle density".into(), Action::Density),
        ("Toggle reduce motion".into(), Action::ReduceMotion),
        ("Toggle player dock".into(), Action::ToggleDock),
        ("Open sources".into(), Action::OpenDialog("sources")),
        ("Open settings".into(), Action::OpenDialog("settings")),
        ("Open profiles".into(), Action::OpenDialog("profiles")),
        ("Open appearance".into(), Action::OpenDialog("appearance")),
    ];
    for (i, p) in profiles.iter().enumerate() {
        v.push((
            format!("Switch to profile {}", p.name),
            Action::SwitchProfile(i),
        ));
    }
    v
}
