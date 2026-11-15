//! What the list does with a row action (ports `ui/row_actions.rs`).
use super::{details, groups, series};
use crate::model::{MediaKind, StreamRow};
use crate::player::PlayerKind;
use crate::slint_ui::state::SlintApp;
use crate::worker::Command;

pub enum RowAction {
    Activate,
    Play(PlayerKind),
    Info,
    CopyUrl,
    ToggleFavorite,
    ToggleHidden,
    AddToGroup(i64),
    NewGroup,
    RemoveFromGroup,
    PlayFrom(i64),
}

pub fn run(a: &mut SlintApp, idx: usize, action: RowAction) {
    let Some(row) = a.streams.row(idx) else {
        return;
    };
    let profile = a.profile();
    let allowed_missing = matches!(
        action,
        RowAction::ToggleFavorite | RowAction::RemoveFromGroup
    );
    if row.missing && !allowed_missing {
        return;
    }
    match action {
        RowAction::Activate if row.media == MediaKind::Series => series::open(a, &row),
        RowAction::Activate => play(a, &row, None),
        RowAction::Play(kind) => play(a, &row, Some(kind)),
        RowAction::Info => details::open(a, &row),
        RowAction::CopyUrl => a.worker.send(Command::CopyUrl { stream_id: row.id }),
        RowAction::ToggleFavorite => {
            let res =
                a.db.set_favorite(profile, row.source_id, row.media, &row.key, !row.favorite);
            a.report(res);
            a.reload_streams();
        }
        RowAction::ToggleHidden => {
            let res =
                a.db.set_stream_hidden(profile, row.source_id, row.media, &row.key, !row.hidden);
            a.report(res);
            a.reload_streams();
        }
        RowAction::AddToGroup(g) => groups::add_to_group(a, g, &row),
        RowAction::NewGroup => {
            let item = Some((row.source_id, row.key.clone()));
            groups::open_group_dialog(a, None, String::new(), item);
        }
        RowAction::RemoveFromGroup => groups::remove_from_group(a, &row),
        RowAction::PlayFrom(id) => {
            a.play_req = Some(id);
            a.worker.send(Command::Play {
                profile,
                stream_id: id,
                kind: None,
            });
        }
    }
}

fn play(a: &mut SlintApp, row: &StreamRow, kind: Option<PlayerKind>) {
    a.play_req = Some(row.id);
    a.worker.send(Command::Play {
        profile: a.profile(),
        stream_id: row.id,
        kind,
    });
}
