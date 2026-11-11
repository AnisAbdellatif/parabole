pub struct Scanned<'a> {
    /// Keys lowercased, in order of appearance.
    pub attrs: Vec<(String, String)>,
    /// Text after the first comma outside quotes.
    pub title: Option<&'a str>,
}

/// Scans `key="v" key2='v' key3=v,Title`. Bare words (like the `-1` duration) are skipped.
pub fn scan(s: &str) -> Scanned<'_> {
    let b = s.as_bytes();
    let mut i = 0;
    let mut attrs = Vec::new();
    while i < b.len() {
        match b[i] {
            c if c.is_ascii_whitespace() => i += 1,
            b',' => {
                return Scanned {
                    attrs,
                    title: Some(s[i + 1..].trim()),
                };
            }
            _ => {
                let start = i;
                while i < b.len() && b[i] != b'=' && b[i] != b',' && !b[i].is_ascii_whitespace() {
                    i += 1;
                }
                let key = &s[start..i];
                if b.get(i) != Some(&b'=') {
                    continue;
                }
                i += 1;
                let value = match b.get(i) {
                    Some(&q) if q == b'"' || q == b'\'' => {
                        let from = i + 1;
                        i = from;
                        while i < b.len() && b[i] != q {
                            i += 1;
                        }
                        let v = &s[from..i];
                        i = (i + 1).min(b.len());
                        v
                    }
                    _ => {
                        let from = i;
                        while i < b.len() && b[i] != b',' && !b[i].is_ascii_whitespace() {
                            i += 1;
                        }
                        &s[from..i]
                    }
                };
                attrs.push((key.to_ascii_lowercase(), value.to_string()));
            }
        }
    }
    Scanned { attrs, title: None }
}

/// First non-empty value for `key`.
pub fn get<'a>(attrs: &'a [(String, String)], key: &str) -> Option<&'a str> {
    attrs
        .iter()
        .find(|(k, v)| k == key && !v.trim().is_empty())
        .map(|(_, v)| v.trim())
}
