//! Content-area chrome: title/subtitle header.
use super::App;
use super::state::Sel;
use super::theme;
use egui::{Align, Frame, Layout, Margin, RichText, Ui};

impl App {
    pub(super) fn list_header(&mut self, ui: &mut Ui) {
        let title = self.list_title();
        let n = self.streams.len();
        let subtitle = match n {
            0 => String::new(),
            1 => "1 item".to_string(),
            n => format!("{n} items"),
        };
        Frame::NONE
            .inner_margin(Margin {
                left: 24,
                right: 24,
                top: 18,
                bottom: 12,
            })
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(title).font(theme::semibold(20.0)));
                    ui.label(
                        RichText::new(subtitle)
                            .font(theme::regular(13.0))
                            .color(theme::TEXT_MUTED),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        self.sort_selector(ui);
                    });
                });
            });
        if !self.search.applied.is_empty() {
            Frame::NONE
                .inner_margin(Margin {
                    left: 24,
                    right: 24,
                    top: 0,
                    bottom: 12,
                })
                .show(ui, |ui| self.search_chips(ui));
        }
    }

    fn list_title(&self) -> String {
        if !self.search.applied.is_empty() {
            return format!("Results for “{}”", self.search.applied);
        }
        match &self.state.sel {
            Some(Sel::Favorites) => "Favorites".into(),
            Some(Sel::Recent) => "Recent".into(),
            Some(Sel::Continue) => "Continue watching".into(),
            Some(Sel::Group(id)) => self
                .groups
                .iter()
                .find(|g| g.id == *id)
                .map_or(String::new(), |g| g.name.clone()),
            Some(Sel::Category(s, k)) => self
                .categories
                .iter()
                .find(|c| c.source_id == *s && c.key == *k)
                .map_or(String::new(), |c| c.name.clone()),
            None => "Select a category".into(),
        }
    }
}
