//! Row context menu (ports `ui/row_menu.rs`, `ui/menu_data.rs`).
use super::rows::{self, RowAction};
use crate::model::{MediaKind, StreamRow};
use crate::player::PlayerKind;
use crate::slint_ui::state::{Sel, SlintApp, with_app};
use crate::slint_ui::{AppWindow, BrowseApi, MenuEntry};
use slint::{ComponentHandle, ModelRc, VecModel};

fn item(id: impl Into<String>, label: impl Into<String>) -> MenuEntry {
    MenuEntry {
        id: id.into().into(),
        label: label.into().into(),
        kind: 0,
    }
}

fn header(label: &str) -> MenuEntry {
    MenuEntry {
        id: "".into(),
        label: label.into(),
        kind: 1,
    }
}

fn separator() -> MenuEntry {
    MenuEntry {
        id: "".into(),
        label: "".into(),
        kind: 2,
    }
}

fn in_group(a: &SlintApp) -> bool {
    matches!(a.state.sel, Some(Sel::Group(_))) && a.search_applied.is_empty()
}

fn entries(a: &mut SlintApp, row: &StreamRow) -> Vec<MenuEntry> {
    let grouped = in_group(a);
    if row.missing {
        return vec![if grouped {
            item("rmgroup", "Remove from group")
        } else {
            item("fav", "Remove from favorites")
        }];
    }
    let mut v = vec![
        item(
            "fav",
            if row.favorite {
                "Unfavorite"
            } else {
                "Favorite"
            },
        ),
        item("hide", if row.hidden { "Unhide" } else { "Hide" }),
    ];
    if row.media == a.state.tab {
        v.push(separator());
        v.push(header("ADD TO GROUP"));
        v.extend(
            a.groups
                .iter()
                .map(|g| item(format!("grp:{}", g.id), &g.name)),
        );
        v.push(item("newgroup", "New group…"));
        if grouped {
            v.push(item("rmgroup", "Remove from group"));
        }
    }
    v.push(if row.media == MediaKind::Live {
        item("info", "TV guide")
    } else {
        item("info", "Info…")
    });
    if row.media == MediaKind::Series {
        return v;
    }
    v.push(separator());
    v.push(header("PLAY WITH"));
    v.extend([
        item("play:mpv", "mpv"),
        item("play:vlc", "VLC"),
        item("play:custom", "Custom"),
    ]);
    if row.media == MediaKind::Live {
        match a.db.alternates(a.profile(), row.id) {
            Ok(alts) if !alts.is_empty() => {
                v.push(header("PLAY FROM"));
                v.extend(
                    alts.iter()
                        .map(|x| item(format!("alt:{}", x.stream_id), &x.source_name)),
                );
            }
            Ok(_) => {}
            Err(e) => a.toast(format!("{e:#}")),
        }
    }
    v.push(separator());
    v.push(item("copy", "Copy stream URL"));
    v
}

fn action(id: &str) -> Option<RowAction> {
    Some(match id {
        "fav" => RowAction::ToggleFavorite,
        "hide" => RowAction::ToggleHidden,
        "info" => RowAction::Info,
        "copy" => RowAction::CopyUrl,
        "newgroup" => RowAction::NewGroup,
        "rmgroup" => RowAction::RemoveFromGroup,
        "play:mpv" => RowAction::Play(PlayerKind::Mpv),
        "play:vlc" => RowAction::Play(PlayerKind::Vlc),
        "play:custom" => RowAction::Play(PlayerKind::Custom),
        _ => {
            if let Some(g) = id.strip_prefix("grp:") {
                RowAction::AddToGroup(g.parse().ok()?)
            } else {
                RowAction::PlayFrom(id.strip_prefix("alt:")?.parse().ok()?)
            }
        }
    })
}

pub fn connect(win: &AppWindow) {
    let api = win.global::<BrowseApi>();
    api.on_row_menu_requested(|i| {
        with_app(|a| {
            let Some(row) = a.streams.row(i as usize) else {
                return;
            };
            a.browse.menu_row = Some(i as usize);
            let v = entries(a, &row);
            if let Some(w) = a.win.upgrade() {
                w.global::<BrowseApi>()
                    .set_row_menu(ModelRc::new(VecModel::from(v)));
            }
        });
    });
    api.on_row_menu_picked(|id| {
        with_app(|a| {
            let (Some(idx), Some(act)) = (a.browse.menu_row.take(), action(&id)) else {
                return;
            };
            rows::run(a, idx, act);
        });
    });
}
