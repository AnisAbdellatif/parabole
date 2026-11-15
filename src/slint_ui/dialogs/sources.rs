//! Sources dialog (mirrors `ui/sources/{mod,list,account,ua_preset}.rs`).
use super::DialogApi;
use super::source_form::Fields;
use super::ua_preset::{PRESETS, is_custom};
use crate::model::SourceKind;
use crate::slint_ui::SourceRow;
use crate::slint_ui::state::SlintApp;
use crate::sources::xtream::{AccountInfo, Expiry, format_date};
use crate::worker::{Command, auto_refresh};
use slint::{ComponentHandle, ModelRc, VecModel};
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) const CUSTOM_UA: i32 = 5;

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64)
}

fn ago(ts: i64, now: i64) -> String {
    match now - ts {
        s if s < 90 => "refreshed just now".into(),
        s if s < 5400 => format!("refreshed {} min ago", s / 60),
        s if s < 129_600 => format!("refreshed {} h ago", s / 3600),
        s => format!("refreshed {} d ago", s / 86_400),
    }
}

/// One-line account summary and its level (0 normal, 1 warning, 2 expired).
fn account_text(a: &AccountInfo, now: i64) -> (String, i32) {
    let expiry = a.expiry_text();
    let (exp, mut level) = match a.expiry(now) {
        Expiry::Expired => (format!("{expiry} (expired)"), 2),
        Expiry::Soon => {
            let days = (a.exp_date.unwrap_or(now) - now) / 86_400;
            (format!("{expiry} (in {days} d)"), 1)
        }
        _ => (expiry, 0),
    };
    if a.connections_full() {
        level = level.max(1);
    }
    let mut t = format!(
        "Status {} · Expires {exp} · Connections {}",
        a.status,
        a.connections_text()
    );
    if a.is_trial {
        t.push_str(" · Trial");
    }
    if let Some(c) = a.created_at {
        t.push_str(&format!(" · Created {}", format_date(c)));
    }
    (t, level)
}

fn preset_index(ua: &str) -> i32 {
    if is_custom(ua) {
        return CUSTOM_UA;
    }
    PRESETS
        .iter()
        .position(|(_, v)| *v == ua.trim())
        .map_or(0, |i| i as i32 + 1)
}

pub(super) fn reset_form(api: &DialogApi) {
    api.set_form_open(false);
    api.set_form_editing(false);
    api.set_form_is_xtream(true);
    for set in [
        DialogApi::set_form_paste,
        DialogApi::set_form_detected,
        DialogApi::set_form_name,
        DialogApi::set_form_url,
        DialogApi::set_form_user,
        DialogApi::set_form_pass,
        DialogApi::set_form_mirrors,
        DialogApi::set_form_ua,
        DialogApi::set_test_text,
    ] {
        set(api, "".into());
    }
    api.set_ua_preset(0);
}

fn read_fields(a: &SlintApp) -> Option<Fields> {
    let w = a.win.upgrade()?;
    let api = w.global::<DialogApi>();
    Some(Fields {
        id: a.dialogs.editing,
        name: api.get_form_name().into(),
        url: api.get_form_url().into(),
        user: api.get_form_user().into(),
        pass: api.get_form_pass().into(),
        mirrors: api.get_form_mirrors().into(),
        ua: api.get_form_ua().into(),
    })
}

impl SlintApp {
    pub(super) fn load_sources_dialog(&mut self) {
        self.dialogs.editing = None;
        self.dialogs.confirm_source = None;
        let hours = auto_refresh::hours(&self.db);
        if let Some(w) = self.win.upgrade() {
            let api = w.global::<DialogApi>();
            reset_form(&api);
            api.set_auto_hours(hours.to_string().into());
        }
        self.push_sources();
    }

    pub(super) fn push_sources(&self) {
        let Some(w) = self.win.upgrade() else { return };
        let api = w.global::<DialogApi>();
        if api.get_open() != "sources" {
            return;
        }
        let now = now();
        let rows: Vec<SourceRow> = self
            .sources
            .iter()
            .map(|s| {
                let xtream = s.kind == SourceKind::Xtream;
                let info = xtream
                    .then(|| self.db.account_json(s.id).ok().flatten())
                    .flatten()
                    .and_then(|j| AccountInfo::parse(&j));
                let (account, account_level) =
                    info.map_or((String::new(), 0), |i| account_text(&i, now));
                SourceRow {
                    id: s.id as i32,
                    name: s.name.as_str().into(),
                    kind: match s.kind {
                        SourceKind::Xtream => "Xtream",
                        SourceKind::M3uUrl => "M3U link",
                        SourceKind::M3uFile => "M3U file",
                    }
                    .into(),
                    refreshed: s
                        .last_refresh
                        .map(|t| ago(t, now))
                        .unwrap_or_default()
                        .into(),
                    refreshing: self.refreshing.contains(&s.id),
                    error: s.last_error.clone().unwrap_or_default().into(),
                    account: account.into(),
                    account_level,
                    xtream,
                    checking: self.dialogs.checking.contains(&s.id),
                    confirm: self.dialogs.confirm_source == Some(s.id),
                }
            })
            .collect();
        api.set_sources(ModelRc::new(VecModel::from(rows)));
    }

    pub(super) fn set_test_result(&mut self, res: Result<String, String>) {
        let Some(w) = self.win.upgrade() else { return };
        let api = w.global::<DialogApi>();
        api.set_testing(false);
        api.set_test_ok(res.is_ok());
        api.set_test_text(res.unwrap_or_else(|e| e).into());
    }

    pub(super) fn load_source_into_form(&mut self, id: i64) {
        let Some(s) = self.sources.iter().find(|s| s.id == id) else {
            return;
        };
        self.dialogs.editing = Some(id);
        let Some(w) = self.win.upgrade() else { return };
        let api = w.global::<DialogApi>();
        reset_form(&api);
        api.set_form_open(true);
        api.set_form_editing(true);
        api.set_form_is_xtream(s.kind == SourceKind::Xtream);
        api.set_form_name(s.name.as_str().into());
        api.set_form_url(s.url.as_str().into());
        api.set_form_user(s.username.clone().unwrap_or_default().into());
        api.set_form_pass(s.password.clone().unwrap_or_default().into());
        api.set_form_mirrors(s.mirrors.join("\n").into());
        let ua = s.user_agent.clone().unwrap_or_default();
        api.set_ua_preset(preset_index(&ua));
        api.set_form_ua(ua.into());
    }

    pub(super) fn submit_source_form(&mut self) {
        let Some(fields) = read_fields(self) else {
            return;
        };
        match fields.build() {
            Ok(src) => {
                let editing = self.dialogs.editing.take();
                if let Some(id) = editing {
                    self.worker.send(Command::UpdateSource(src));
                    self.worker.send(Command::Refresh(id));
                } else {
                    self.worker.send(Command::AddSource(self.profile(), src));
                }
                if let Some(w) = self.win.upgrade() {
                    reset_form(&w.global::<DialogApi>());
                }
            }
            Err(e) => self.toast(e),
        }
    }

    pub(super) fn test_source_form(&mut self) {
        let Some(fields) = read_fields(self) else {
            return;
        };
        match fields.build() {
            Ok(src) => {
                if let Some(w) = self.win.upgrade() {
                    let api = w.global::<DialogApi>();
                    api.set_testing(true);
                    api.set_test_text("".into());
                }
                self.worker.send(Command::TestSource(src));
            }
            Err(e) => self.toast(e),
        }
    }
}
