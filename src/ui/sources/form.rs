use super::SourcesDialog;
use super::ua_preset::{is_custom, ua_row};
use crate::model::{Source, SourceInput, SourceKind};
use crate::sources::detect::detect;
use crate::ui::theme::{primary_button, secondary_button};
use crate::worker::Command;

impl SourcesDialog {
    pub(super) fn load(&mut self, s: &Source) {
        self.reset_form();
        *self = SourcesDialog {
            editing: Some(s.id),
            name: s.name.clone(),
            url: s.url.clone(),
            user: s.username.clone().unwrap_or_default(),
            pass: s.password.clone().unwrap_or_default(),
            ua: s.user_agent.clone().unwrap_or_default(),
            ua_custom: is_custom(s.user_agent.as_deref().unwrap_or_default()),
            mirrors: s.mirrors.join("\n"),
            checking: std::mem::take(&mut self.checking),
            auto_hours: self.auto_hours,
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

pub(super) fn source_form(
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
            ua_row(ui, &mut d.ua, &mut d.ua_custom);
            ui.end_row();
        });
    let mut error = None;
    ui.horizontal(|ui| {
        let label = if d.editing.is_some() { "Save" } else { "Add" };
        if primary_button(ui, label).clicked() {
            match d.build() {
                Ok(src) if d.editing.is_some() => {
                    cmds.push(Command::UpdateSource(src));
                    cmds.push(Command::Refresh(d.editing.unwrap_or_default()));
                    d.reset_form();
                }
                Ok(src) => {
                    cmds.push(Command::AddSource(profile, src));
                    d.reset_form();
                }
                Err(e) => error = Some(e),
            }
        }
        if d.editing.is_some() && secondary_button(ui, "Cancel edit").clicked() {
            d.reset_form();
        }
        if ui
            .add_enabled(!d.testing, egui::Button::new("Test"))
            .clicked()
        {
            match d.build() {
                Ok(src) => {
                    d.testing = true;
                    d.test_result = None;
                    cmds.push(Command::TestSource(src));
                }
                Err(e) => error = Some(e),
            }
        }
        if d.testing {
            ui.spinner();
        }
    });
    match &d.test_result {
        Some(Ok(msg)) => ui.colored_label(egui::Color32::LIGHT_GREEN, msg),
        Some(Err(e)) => ui.colored_label(egui::Color32::LIGHT_RED, e),
        None => ui.label(""),
    };
    error
}
