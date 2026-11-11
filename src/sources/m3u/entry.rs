use super::UNCATEGORIZED;
use super::attrs::{self, Scanned};
use crate::model::{Headers, MediaKind, ParsedStream};

/// Metadata collected from the lines preceding a URL line.
#[derive(Default)]
pub struct Pending {
    title: Option<String>,
    tvg_id: Option<String>,
    tvg_name: Option<String>,
    group: Option<String>,
    extgrp: Option<String>,
    logo: Option<String>,
    num: Option<i64>,
    archive_days: Option<i64>,
    headers: Headers,
    drm: bool,
}

/// A stream whose final `ext_key` is not yet known (needs the whole playlist).
pub struct Draft {
    pub stream: ParsedStream,
    pub tvg_id: Option<String>,
    pub group: String,
}

impl Pending {
    pub fn apply_extinf(&mut self, rest: &str) {
        let Scanned { attrs, title } = attrs::scan(rest);
        let text = |key| attrs::get(&attrs, key).map(str::to_string);
        self.title = title.filter(|t| !t.is_empty()).map(str::to_string);
        self.tvg_id = text("tvg-id");
        self.tvg_name = text("tvg-name");
        self.logo = text("tvg-logo");
        self.group = attrs::get(&attrs, "group-title")
            .and_then(|g| g.split(';').map(str::trim).find(|g| !g.is_empty()))
            .map(str::to_string);
        let int = |key| attrs::get(&attrs, key).and_then(|v| v.parse().ok());
        self.num = int("tvg-chno").or_else(|| int("channel-number"));
        self.archive_days = int("catchup-days");
    }

    pub fn set_extgrp(&mut self, group: &str) {
        self.extgrp = Some(group.trim().to_string()).filter(|g| !g.is_empty());
    }

    pub fn apply_vlcopt(&mut self, opt: &str) {
        let Some((key, value)) = opt.split_once('=') else {
            return;
        };
        set_header(&mut self.headers, key, value);
    }

    pub fn mark_drm(&mut self) {
        self.drm = true;
    }

    pub fn into_draft(mut self, line: &str) -> Draft {
        let (url, pipe) = match line.split_once('|') {
            Some((url, pipe)) => (url.trim(), pipe),
            None => (line, ""),
        };
        for pair in pipe.split('&') {
            if let Some((key, value)) = pair.split_once('=') {
                set_header(&mut self.headers, key, value);
            }
        }
        let name = [&self.title, &self.tvg_name]
            .into_iter()
            .flatten()
            .next()
            .cloned()
            .unwrap_or_else(|| url.to_string());
        let group = self
            .group
            .or(self.extgrp)
            .unwrap_or_else(|| UNCATEGORIZED.to_string());
        let stream = ParsedStream {
            media: classify(url),
            name,
            num: self.num,
            logo: self.logo,
            url: Some(url.to_string()),
            epg_id: self.tvg_id.clone(),
            archive_days: self.archive_days,
            headers: self.headers,
            extra_json: self.drm.then(|| r#"{"drm":true}"#.to_string()),
            category_key: Some(group.clone()),
            ..Default::default()
        };
        Draft {
            stream,
            tvg_id: self.tvg_id,
            group,
        }
    }
}

fn set_header(headers: &mut Headers, key: &str, value: &str) {
    let value = Some(value.trim().to_string()).filter(|v| !v.is_empty());
    match key.trim().to_ascii_lowercase().as_str() {
        "http-user-agent" | "user-agent" => headers.user_agent = value,
        "http-referrer" | "http-referer" | "referer" | "referrer" => headers.referrer = value,
        "http-origin" | "origin" => headers.origin = value,
        _ => {}
    }
}

fn classify(url: &str) -> MediaKind {
    let path = url
        .split(['?', '#'])
        .next()
        .unwrap_or(url)
        .to_ascii_lowercase();
    if path.contains("/movie/") {
        return MediaKind::Movie;
    }
    if path.contains("/series/") {
        return MediaKind::Series;
    }
    match path.rsplit_once('.') {
        Some((_, "mp4" | "mkv" | "avi")) => MediaKind::Movie,
        _ => MediaKind::Live,
    }
}
