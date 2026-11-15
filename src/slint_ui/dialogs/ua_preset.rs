//! User-agent preset picker for the source form.

pub(crate) const PRESETS: [(&str, &str); 4] = [
    ("IPTV Smarters Pro", "IPTVSmartersPro"),
    ("TiviMate", "TiviMate/4.7.0 (Android 11)"),
    ("OkHttp", "okhttp/4.9.0"),
    ("VLC", "VLC/3.0.20 LibVLC/3.0.20"),
];

/// Whether `ua` must be edited as free text (neither empty nor a preset).
pub(crate) fn is_custom(ua: &str) -> bool {
    let ua = ua.trim();
    !ua.is_empty() && !PRESETS.iter().any(|(_, v)| *v == ua)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_detection() {
        assert!(!is_custom(""));
        assert!(!is_custom("okhttp/4.9.0"));
        assert!(is_custom("MyAgent/1"));
    }
}
