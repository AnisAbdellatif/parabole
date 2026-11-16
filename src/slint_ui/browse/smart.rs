//! Smart tab: quality-bucket sidebar, channel rows and quality badges.
use crate::model::{MediaKind, StreamRow};
use crate::slint_ui::state::{Sel, SlintApp};
use crate::slint_ui::{Core, SideRow};
use crate::smart::{Bucket, badge, quality_label};
use slint::ComponentHandle;
use std::collections::HashMap;

fn row(id: String, kind: i32, label: &str, count: i32) -> SideRow {
    SideRow {
        id: id.into(),
        kind,
        label: label.into(),
        count,
        icon: 0,
        pinned: false,
    }
}

impl SlintApp {
    pub fn push_smart_sidebar(&self) {
        let Some(win) = self.win.upgrade() else {
            return;
        };
        let mut v = vec![row(String::new(), 0, "QUALITY", -1)];
        for b in Bucket::ALL {
            let n = self.smart.iter().filter(|c| b.contains(c)).count();
            if n > 0 || b == Bucket::All {
                v.push(row(Sel::Quality(b.index()).to_id(), 1, b.label(), n as i32));
            }
        }
        let sel_id = self.state.sel.as_ref().map(Sel::to_id).unwrap_or_default();
        let at = v
            .iter()
            .position(|r| !sel_id.is_empty() && r.id == sel_id.as_str());
        self.browse.side_rows.set_vec(v);
        let core = win.global::<Core>();
        core.set_selected_index(at.map_or(-1, |i| i as i32));
        core.set_selected_id(sel_id.into());
        self.push_edit();
    }

    /// One row per channel in the bucket: its best variant, under the channel's name.
    pub fn smart_rows(&self, b: u8) -> Vec<StreamRow> {
        let Some(b) = Bucket::from_index(b) else {
            return Vec::new();
        };
        let channels = self.smart.iter().filter(|c| b.contains(c));
        channels
            .map(|c| StreamRow {
                name: c.name.clone(),
                ..c.variants[0].row.clone()
            })
            .collect()
    }

    /// Smart rows get their channel badge; other live rows their measured quality.
    pub fn row_badges(&self, rows: &[StreamRow]) -> HashMap<i64, String> {
        let smart_list =
            self.search_applied.is_empty() && matches!(self.state.sel, Some(Sel::Quality(_)));
        if self.state.smart && smart_list {
            return self
                .smart
                .iter()
                .map(|c| (c.variants[0].row.id, badge(c)))
                .collect();
        }
        let ids: Vec<i64> = rows
            .iter()
            .filter(|r| r.media == MediaKind::Live)
            .map(|r| r.id)
            .collect();
        if ids.is_empty() {
            return HashMap::new();
        }
        // Badges are a hint; a failed lookup just shows none.
        let quality = self.db.quality_by_stream(&ids).unwrap_or_default();
        quality
            .iter()
            .filter_map(|(id, q)| Some((*id, quality_label(q)?)))
            .collect()
    }

    /// Next variant of the failed stream's Smart channel, if any.
    pub fn smart_next_variant(&self, failed: i64) -> Option<&StreamRow> {
        self.smart.iter().find_map(|c| {
            let at = c.variants.iter().position(|v| v.row.id == failed)?;
            c.variants.get(at + 1).map(|v| &v.row)
        })
    }
}
