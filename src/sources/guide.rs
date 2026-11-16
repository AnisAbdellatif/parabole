//! Which XMLTV guide URLs a source uses.
use super::xtream;
use crate::model::{Source, SourceKind};
use anyhow::Result;

/// The playlist's `url-tvg` list, or the Xtream `xmltv.php` URL when it has none.
pub fn guide_urls(source: &Source, epg_url: Option<&str>) -> Result<Vec<String>> {
    let urls: Vec<String> = epg_url
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|u| !u.is_empty())
        .map(String::from)
        .collect();
    if urls.is_empty() && source.kind == SourceKind::Xtream {
        return Ok(vec![xtream::urls::xmltv(source)?]);
    }
    Ok(urls)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn src(kind: SourceKind) -> Source {
        Source {
            id: 1,
            name: "s".into(),
            kind,
            url: "http://h.tv".into(),
            username: Some("u".into()),
            password: Some("p".into()),
            user_agent: None,
            mirrors: Vec::new(),
            output_format: None,
            last_refresh: None,
            last_error: None,
        }
    }

    #[test]
    fn m3u_list_is_split_and_trimmed() {
        let s = src(SourceKind::M3uUrl);
        assert_eq!(
            guide_urls(&s, Some(" http://a/x.xml, ,http://b/y.xml ")).unwrap(),
            ["http://a/x.xml", "http://b/y.xml"]
        );
        assert!(guide_urls(&s, None).unwrap().is_empty());
    }

    #[test]
    fn xtream_falls_back_to_xmltv_but_keeps_explicit_list() {
        let s = src(SourceKind::Xtream);
        assert_eq!(
            guide_urls(&s, Some("")).unwrap(),
            ["http://h.tv/xmltv.php?username=u&password=p"]
        );
        assert_eq!(
            guide_urls(&s, Some("http://a/x.xml")).unwrap(),
            ["http://a/x.xml"]
        );
    }
}
