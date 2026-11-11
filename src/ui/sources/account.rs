use crate::model::{SourceId, SourceKind};
use crate::sources::xtream::{AccountInfo, Expiry, format_date};
use crate::ui::App;
use egui::Color32;

impl App {
    /// Re-reads the stored account info of every Xtream source.
    pub(in crate::ui) fn reload_accounts(&mut self) {
        let db = &self.db;
        self.accounts = self
            .sources
            .iter()
            .filter(|s| s.kind == SourceKind::Xtream)
            .filter_map(|s| {
                let json = db.account_json(s.id).ok().flatten()?;
                Some((s.id, AccountInfo::parse(&json)?))
            })
            .collect();
    }

    /// Warns (never blocks) when the last known account state has no free connection.
    pub(in crate::ui) fn warn_if_connections_full(&mut self, source: SourceId) {
        if let Some(a) = self.accounts.get(&source).filter(|a| a.connections_full()) {
            let text = format!(
                "All connections in use ({}): the provider may refuse this stream",
                a.connections_text()
            );
            self.toast(text);
        }
    }
}

pub(super) fn panel(ui: &mut egui::Ui, a: &AccountInfo, now: i64) {
    ui.horizontal_wrapped(|ui| {
        ui.weak("Status");
        ui.label(&a.status);
        ui.weak("Expires");
        let expiry = a.expiry_text();
        match a.expiry(now) {
            Expiry::Expired => ui.colored_label(Color32::LIGHT_RED, format!("{expiry} (expired)")),
            Expiry::Soon => {
                let days = (a.exp_date.unwrap_or(now) - now) / 86_400;
                ui.colored_label(Color32::ORANGE, format!("{expiry} (in {days} d)"))
            }
            _ => ui.label(expiry),
        };
        ui.weak("Connections");
        let conns = a.connections_text();
        if a.connections_full() {
            ui.colored_label(Color32::ORANGE, conns);
        } else {
            ui.label(conns);
        }
        if a.is_trial {
            ui.label("Trial");
        }
        if let Some(t) = a.created_at {
            ui.weak("Created");
            ui.label(format_date(t));
        }
    });
}
