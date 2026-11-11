//! User-agent preset picker for the source form.

const PRESETS: [(&str, &str); 4] = [
    ("IPTV Smarters Pro", "IPTVSmartersPro"),
    ("TiviMate", "TiviMate/4.7.0 (Android 11)"),
    ("OkHttp", "okhttp/4.9.0"),
    ("VLC", "VLC/3.0.20 LibVLC/3.0.20"),
];

/// Whether `ua` must be edited as free text (neither empty nor a preset).
pub(super) fn is_custom(ua: &str) -> bool {
    let ua = ua.trim();
    !ua.is_empty() && !PRESETS.iter().any(|(_, v)| *v == ua)
}

fn label(ua: &str, custom: bool) -> &'static str {
    if custom {
        return "Custom…";
    }
    PRESETS
        .iter()
        .find(|(_, v)| *v == ua.trim())
        .map_or("Default", |(name, _)| name)
}

pub(super) fn ua_row(ui: &mut egui::Ui, ua: &mut String, custom: &mut bool) {
    ui.horizontal(|ui| {
        egui::ComboBox::from_id_salt("ua_preset")
            .selected_text(label(ua, *custom))
            .show_ui(ui, |ui| {
                if ui
                    .selectable_label(!*custom && ua.is_empty(), "Default")
                    .clicked()
                {
                    (*custom, *ua) = (false, String::new());
                }
                for (name, value) in PRESETS {
                    let selected = !*custom && ua.trim() == value;
                    if ui.selectable_label(selected, name).clicked() {
                        (*custom, *ua) = (false, value.to_string());
                    }
                }
                if ui.selectable_label(*custom, "Custom…").clicked() {
                    *custom = true;
                }
            });
        if *custom {
            ui.text_edit_singleline(ua);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_detection() {
        assert!(!is_custom(""));
        assert!(!is_custom("okhttp/4.9.0"));
        assert!(is_custom("MyAgent/1"));
        assert_eq!(label("TiviMate/4.7.0 (Android 11)", false), "TiviMate");
        assert_eq!(label("", false), "Default");
    }
}
