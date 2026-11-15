//! Lazy model of poster-grid rows: row `i` holds tiles `i*cols..(i+1)*cols` of the stream list.
use crate::slint_ui::stream_model::StreamModel;
use crate::slint_ui::{GridRow, TileItem};
use slint::{Model, ModelNotify, ModelRc, ModelTracker, VecModel};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

#[derive(Default)]
pub struct GridModel {
    streams: RefCell<Option<Rc<StreamModel>>>,
    cols: Cell<usize>,
    notify: ModelNotify,
}

impl GridModel {
    pub fn attach(&self, streams: Rc<StreamModel>) {
        *self.streams.borrow_mut() = Some(streams);
    }

    fn cols(&self) -> usize {
        self.cols.get().max(1)
    }

    pub fn set_cols(&self, n: usize) {
        if n != self.cols.get() {
            self.cols.set(n);
            self.notify.reset();
        }
    }

    pub fn reset(&self) {
        self.notify.reset();
    }

    /// A poster finished loading for the stream at `stream_idx`.
    pub fn stream_changed(&self, stream_idx: usize) {
        self.notify.row_changed(stream_idx / self.cols());
    }
}

impl Model for GridModel {
    type Data = GridRow;

    fn row_count(&self) -> usize {
        let n = self.streams.borrow().as_ref().map_or(0, |s| s.len());
        n.div_ceil(self.cols())
    }

    fn row_data(&self, i: usize) -> Option<GridRow> {
        let streams = self.streams.borrow();
        let s = streams.as_ref()?;
        let cols = self.cols();
        let end = ((i + 1) * cols).min(s.len());
        let tiles: Vec<TileItem> = (i * cols..end).filter_map(|j| s.tile(j)).collect();
        Some(GridRow {
            tiles: ModelRc::new(VecModel::from(tiles)),
        })
    }

    fn model_tracker(&self) -> &dyn ModelTracker {
        &self.notify
    }
}
