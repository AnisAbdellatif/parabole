//! Lazy list model: rows stay as `StreamRow`; `StreamItem`s are built per visible row.
use super::StreamItem;
use super::images::Images;
use crate::db::progress::{Watch, classify};
use crate::model::{SourceId, StreamRow};
use slint::{Model, ModelNotify, ModelTracker};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

pub struct StreamModel {
    rows: RefCell<Vec<StreamRow>>,
    notify: ModelNotify,
    images: Rc<Images>,
    show_source: Cell<bool>,
    source_names: RefCell<HashMap<SourceId, String>>,
}

fn initials(name: &str) -> String {
    name.split_whitespace()
        .take(2)
        .filter_map(|w| w.chars().next())
        .flat_map(char::to_uppercase)
        .collect()
}

impl StreamModel {
    pub fn new(images: Rc<Images>) -> Self {
        Self {
            rows: Default::default(),
            notify: Default::default(),
            images,
            show_source: Cell::new(false),
            source_names: Default::default(),
        }
    }

    pub fn set_rows(
        &self,
        rows: Vec<StreamRow>,
        show_source: bool,
        names: HashMap<SourceId, String>,
    ) {
        *self.rows.borrow_mut() = rows;
        self.show_source.set(show_source);
        *self.source_names.borrow_mut() = names;
        self.notify.reset();
    }

    pub fn row(&self, i: usize) -> Option<StreamRow> {
        self.rows.borrow().get(i).cloned()
    }

    pub fn image_ready(&self, url: &str) {
        for (i, r) in self.rows.borrow().iter().enumerate() {
            if r.logo.as_deref() == Some(url) {
                self.notify.row_changed(i);
            }
        }
    }

    fn meta(&self, r: &StreamRow) -> String {
        if r.missing {
            return "No longer listed by the provider".into();
        }
        let mut parts = Vec::new();
        if self.show_source.get()
            && let Some(n) = self.source_names.borrow().get(&r.source_id)
        {
            parts.push(n.clone());
        }
        if let Some(n) = r.num {
            parts.push(format!("Channel {n}"));
        }
        parts.join(" · ")
    }
}

impl Model for StreamModel {
    type Data = StreamItem;

    fn row_count(&self) -> usize {
        self.rows.borrow().len()
    }

    fn row_data(&self, i: usize) -> Option<StreamItem> {
        let rows = self.rows.borrow();
        let r = rows.get(i)?;
        let logo = r.logo.as_deref().and_then(|u| self.images.get(u));
        Some(StreamItem {
            name: r.name.as_str().into(),
            meta: self.meta(r).into(),
            initials: initials(&r.name).into(),
            has_logo: logo.is_some(),
            logo: logo.unwrap_or_default(),
            favorite: r.favorite,
            progress: if classify(r.progress) == Watch::Partial {
                r.progress.unwrap_or(0.0)
            } else {
                0.0
            },
            watched: classify(r.progress) == Watch::Watched,
            missing: r.missing,
            hidden: r.hidden,
        })
    }

    fn model_tracker(&self) -> &dyn ModelTracker {
        &self.notify
    }
}

#[cfg(test)]
mod tests {
    use super::initials;

    #[test]
    fn initials_take_two_words() {
        assert_eq!(initials("bbc one hd"), "BO");
        assert_eq!(initials("CNN"), "C");
    }
}
