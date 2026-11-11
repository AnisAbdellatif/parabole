//! What the list does with a `RowAction` raised by a row or card.
use super::App;
use super::state::SeriesState;
use super::stream_row::RowAction;
use crate::model::{MediaKind, SourceKind, StreamRow};
use crate::worker::Command;

impl App {
    pub(super) fn row_action(&mut self, idx: usize, action: RowAction) {
        let Some(row) = self.streams.get(idx).cloned() else {
            return;
        };
        let profile = self.profile();
        if row.missing && !matches!(action, RowAction::ToggleFavorite | RowAction::Select) {
            return;
        }
        match action {
            RowAction::Select => self.selected = idx,
            RowAction::Activate if self.state.tab == MediaKind::Series => self.open_series(&row),
            RowAction::Activate => self.play(&row, None),
            RowAction::Play(kind) => self.play(&row, Some(kind)),
            RowAction::Info => self.open_details(&row),
            RowAction::CopyUrl => self.worker.send(Command::CopyUrl { stream_id: row.id }),
            RowAction::ToggleFavorite => {
                let res = self.db.set_favorite(
                    profile,
                    row.source_id,
                    row.media,
                    &row.key,
                    !row.favorite,
                );
                self.report(res);
                self.reload_streams();
            }
            RowAction::ToggleHidden => {
                let res = self.db.set_stream_hidden(
                    profile,
                    row.source_id,
                    row.media,
                    &row.key,
                    !row.hidden,
                );
                self.report(res);
                self.reload_streams();
            }
        }
    }

    fn play(&mut self, row: &StreamRow, kind: Option<crate::player::PlayerKind>) {
        self.warn_if_connections_full(row.source_id);
        self.worker.send(Command::Play {
            profile: self.profile(),
            stream_id: row.id,
            kind,
        });
    }

    fn open_series(&mut self, row: &StreamRow) {
        let xtream = self
            .sources
            .iter()
            .any(|s| s.id == row.source_id && s.kind == SourceKind::Xtream);
        if !xtream {
            self.toast("Episode lists need an Xtream source");
            return;
        }
        self.series = Some(SeriesState {
            source: row.source_id,
            key: row.key.clone(),
            title: row.name.clone(),
            loading: true,
            seasons: Vec::new(),
            season: 0,
            autoplay: row.resume_episode.as_ref().map(|e| e.id.clone()),
            progress: Default::default(),
            progress_at: None,
        });
        self.worker.send(Command::LoadSeries {
            source: row.source_id,
            key: row.key.clone(),
        });
    }
}
