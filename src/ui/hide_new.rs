use super::App;

impl App {
    pub(super) fn hide_new_toggle(&mut self, ui: &mut egui::Ui) {
        let Some(source) = self.single_source() else {
            return;
        };
        let profile = self.profile();
        let mut on = self
            .db
            .hide_new_categories(profile, source)
            .unwrap_or(false);
        if ui
            .checkbox(&mut on, "Hide new categories")
            .on_hover_text("Categories that appear after a refresh start hidden")
            .changed()
        {
            let res = self.db.set_hide_new_categories(profile, source, on);
            self.report(res);
        }
    }
}
