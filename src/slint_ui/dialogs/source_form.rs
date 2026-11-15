//! Add/edit source form: paste detection and building a `Source` (mirrors `ui/sources/form.rs`).
use crate::model::{Source, SourceId, SourceInput, SourceKind};
use crate::sources::detect::detect;

#[derive(Default)]
pub struct Fields {
    pub id: Option<SourceId>,
    pub name: String,
    pub url: String,
    pub user: String,
    pub pass: String,
    pub mirrors: String,
    pub ua: String,
}

/// Status line, whether the Xtream-only fields apply (true when unknown), and the (url, user, pass) to fill in.
pub fn detect_paste(paste: &str) -> (String, bool, Option<(String, String, String)>) {
    match detect(paste) {
        Ok(SourceInput::Xtream {
            base,
            username,
            password,
        }) => (
            format!("Detected: Xtream account “{username}” on {base}"),
            true,
            Some((base, username, password)),
        ),
        Ok(SourceInput::M3uUrl(u)) => (
            "Detected: M3U playlist URL".into(),
            false,
            Some((u, String::new(), String::new())),
        ),
        Ok(SourceInput::M3uFile(p)) => (
            "Detected: M3U file".into(),
            false,
            Some((p, String::new(), String::new())),
        ),
        Err(e) => (format!("Not recognized: {e}"), true, None),
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

impl Fields {
    fn mirrors(&self, kind: SourceKind) -> Result<Vec<String>, String> {
        if kind != SourceKind::Xtream {
            return Ok(Vec::new());
        }
        self.mirrors
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(
                |l| match detect(&format!("{l}\n{}\n{}", self.user.trim(), self.pass)) {
                    Ok(SourceInput::Xtream { base, .. }) => Ok(base),
                    _ => Err(format!("Invalid backup URL: {l}")),
                },
            )
            .collect()
    }

    pub fn build(&self) -> Result<Source, String> {
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
        let mirrors = self.mirrors(kind)?;
        let name = match self.name.trim() {
            "" => default_name(&url),
            n => n.to_string(),
        };
        let ua = self.ua.trim();
        Ok(Source {
            id: self.id.unwrap_or_default(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_m3u_with_host_as_default_name() {
        let f = Fields {
            url: "http://example.com/list.m3u".into(),
            ua: " okhttp/4.9.0 ".into(),
            ..Default::default()
        };
        let s = f.build().expect("valid");
        assert_eq!(s.kind, SourceKind::M3uUrl);
        assert_eq!(s.name, "example.com");
        assert_eq!(s.user_agent.as_deref(), Some("okhttp/4.9.0"));
    }

    #[test]
    fn builds_xtream_and_rejects_bad_input() {
        let f = Fields {
            url: "http://host:8080".into(),
            user: "u".into(),
            pass: "p".into(),
            ..Default::default()
        };
        assert_eq!(f.build().expect("valid").kind, SourceKind::Xtream);
        assert!(Fields::default().build().is_err());
    }
}
