use super::App;
use crate::model::{Source, SourceId, SourceInput, SourceKind};
use crate::sources::detect::detect;
use crate::worker::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Default)]
pub struct SourcesDialog {
    editing: Option<SourceId>,
    paste: String,
    detected: String,
    name: String,
    url: String,
    user: String,
    pass: String,
    ua: String,
    mirrors: String,
    confirm_delete: Option<SourceId>,
}

impl SourcesDialog {
    fn load(&mut self, s: &Source) {
        *self = SourcesDialog {
            editing: Some(s.id),
            name: s.name.clone(),
            url: s.url.clone(),
            user: s.username.clone().unwrap_or_default(),
            pass: s.password.clone().unwrap_or_default(),
            ua: s.user_agent.clone().unwrap_or_default(),
            mirrors: s.mirrors.join("\n"),
            ..Default::default()
        };
    }

    fn apply_paste(&mut self) {
        match detect(&self.paste) {
            Ok(SourceInput::Xtream {
                base,
                username,
                password,
            }) => {
                self.detected = format!("Detected: Xtream account “{username}” on {base}");
                (self.url, self.user, self.pass) = (base, username, password);
            }
            Ok(SourceInput::M3uUrl(u)) => {
                self.detected = "Detected: M3U playlist URL".into();
                (self.url, self.user, self.pass) = (u, String::new(), String::new());
            }
            Ok(SourceInput::M3uFile(p)) => {
                self.detected = "Detected: M3U file".into();
                (self.url, self.user, self.pass) = (p, String::new(), String::new());
            }
            Err(e) => self.detected = format!("Not recognized: {e}"),
        }
    }

    fn build_mirrors(&self, kind: SourceKind) -> Result<Vec<String>, String> {
        let lines = self
            .mirrors
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty());
        if kind != SourceKind::Xtream {
            return Ok(Vec::new());
        }
        lines
            .map(
                |l| match detect(&format!("{l}\n{}\n{}", self.user.trim(), self.pass)) {
                    Ok(SourceInput::Xtream { base, .. }) => Ok(base),
                    _ => Err(format!("Invalid backup URL: {l}")),
                },
            )
            .collect()
    }

    fn build(&self) -> Result<Source, String> {
        let input = if self.user.trim().is_empty() {
            self.url.trim().to_string()
        } else {
            format!("{}\n{}\n{}", self.url.trim(), self.user.trim(), self.pass)
        };
        let (kind, url, username, password) = match detect(&input).map_err(|e| e.to_string())? {
            SourceInput::Xtream {
                base,
                username,
                password,
            } => (SourceKind::Xtream, base, Some(username), Some(password)),
            SourceInput::M3uUrl(u) => (SourceKind::M3uUrl, u, None, None),
            SourceInput::M3uFile(p) => (SourceKind::M3uFile, p, None, None),
        };
        let mirrors = self.build_mirrors(kind)?;
        let name = match self.name.trim() {
            "" => default_name(&url),
            n => n.to_string(),
        };
        let ua = self.ua.trim();
        Ok(Source {
            id: self.editing.unwrap_or_default(),
            name,
            kind,
            url,
            mirrors,
            username,
            password,
            user_agent: (!ua.is_empty()).then(|| ua.to_string()),
            output_format: None,
            last_refresh: None,
            last_error: None,
        })
    }
}

fn default_name(url: &str) -> String {
    url::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(str::to_string))
        .or_else(|| {
            std::path::Path::new(url)
                .file_name()
                .map(|f| f.to_string_lossy().into_owned())
        })
        .unwrap_or_else(|| "Source".into())
}

fn ago(ts: i64) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64);
    match now - ts {
        s if s < 90 => "just now".into(),
        s if s < 5400 => format!("{} min ago", s / 60),
        s if s < 129_600 => format!("{} h ago", s / 3600),
        s => format!("{} d ago", s / 86_400),
    }
}

impl App {
    pub(super) fn open_sources(&mut self) {
        self.sources_dialog.get_or_insert_with(Default::default);
    }

    pub(super) fn sources_window(&mut self, ctx: &egui::Context) {
        let Some(mut d) = self.sources_dialog.take() else {
            return;
        };
        let mut open = true;
        let mut cmds = Vec::new();
        let mut error = None;
        egui::Window::new("Sources")
            .open(&mut open)
            .default_width(560.0)
            .show(ctx, |ui| {
                self.source_list(ui, &mut d, &mut cmds);
                ui.separator();
                ui.heading(if d.editing.is_some() {
                    "Edit source"
                } else {
                    "Add source"
                });
                error = source_form(ui, &mut d, &mut cmds, self.profile());
            });
        for c in cmds {
            self.worker.send(c);
        }
        if let Some(e) = error {
            self.toast(e);
        }
        if open {
            self.sources_dialog = Some(d);
        }
    }

    fn source_list(&self, ui: &mut egui::Ui, d: &mut SourcesDialog, cmds: &mut Vec<Command>) {
        if self.sources.is_empty() {
            ui.weak("No sources yet.");
        }
        for s in &self.sources {
            ui.horizontal(|ui| {
                ui.strong(&s.name);
                ui.weak(s.kind.as_str());
                if self.refreshing.contains(&s.id) {
                    ui.spinner();
                } else if let Some(ts) = s.last_refresh {
                    ui.weak(format!("refreshed {}", ago(ts)));
                }
                if ui
                    .add_enabled(
                        !self.refreshing.contains(&s.id),
                        egui::Button::new("Refresh"),
                    )
                    .clicked()
                {
                    cmds.push(Command::Refresh(s.id));
                }
                if ui.button("Edit").clicked() {
                    d.load(s);
                }
                if d.confirm_delete == Some(s.id) {
                    if ui.button("Really delete").clicked() {
                        cmds.push(Command::DeleteSource(s.id));
                        d.confirm_delete = None;
                    }
                    if ui.button("Cancel").clicked() {
                        d.confirm_delete = None;
                    }
                } else if ui.button("Delete").clicked() {
                    d.confirm_delete = Some(s.id);
                }
            });
            if let Some(e) = &s.last_error {
                ui.colored_label(egui::Color32::LIGHT_RED, e);
            }
        }
        if self.sources.len() > 1 && ui.button("Refresh all").clicked() {
            cmds.push(Command::RefreshAll(self.profile()));
        }
    }
}

fn source_form(
    ui: &mut egui::Ui,
    d: &mut SourcesDialog,
    cmds: &mut Vec<Command>,
    profile: i64,
) -> Option<String> {
    ui.label("Paste anything (M3U link, Xtream URL + username + password, file path):");
    if ui
        .add(
            egui::TextEdit::multiline(&mut d.paste)
                .desired_rows(3)
                .desired_width(f32::INFINITY),
        )
        .changed()
    {
        d.apply_paste();
    }
    if !d.detected.is_empty() {
        ui.weak(&d.detected);
    }
    egui::Grid::new("source_form")
        .num_columns(2)
        .show(ui, |ui| {
            ui.label("Name");
            ui.text_edit_singleline(&mut d.name);
            ui.end_row();
            ui.label("URL / file");
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut d.url);
                if ui.button("Choose file…").clicked() {
                    let picked = rfd::FileDialog::new()
                        .add_filter("Playlist", &["m3u", "m3u8"])
                        .pick_file();
                    if let Some(p) = picked {
                        d.url = p.to_string_lossy().into_owned();
                    }
                }
            });
            ui.end_row();
            ui.label("Backup URLs");
            ui.add(
                egui::TextEdit::multiline(&mut d.mirrors)
                    .desired_rows(2)
                    .hint_text("Xtream only, one per line"),
            );
            ui.end_row();
            ui.label("Username");
            ui.text_edit_singleline(&mut d.user);
            ui.end_row();
            ui.label("Password");
            ui.add(egui::TextEdit::singleline(&mut d.pass).password(true));
            ui.end_row();
            ui.label("User agent");
            ui.text_edit_singleline(&mut d.ua);
            ui.end_row();
        });
    let mut error = None;
    ui.horizontal(|ui| {
        let label = if d.editing.is_some() { "Save" } else { "Add" };
        if ui.button(label).clicked() {
            match d.build() {
                Ok(src) if d.editing.is_some() => {
                    cmds.push(Command::UpdateSource(src));
                    cmds.push(Command::Refresh(d.editing.unwrap_or_default()));
                    *d = SourcesDialog::default();
                }
                Ok(src) => {
                    cmds.push(Command::AddSource(profile, src));
                    *d = SourcesDialog::default();
                }
                Err(e) => error = Some(e),
            }
        }
        if d.editing.is_some() && ui.button("Cancel edit").clicked() {
            *d = SourcesDialog::default();
        }
    });
    error
}
