use crate::model::PlayRequest;
use crate::player::{PlayerSettings, live};
use anyhow::{Result, bail};
use std::ffi::OsString;
use std::path::Path;

/// mpv splits list options on commas, so a comma inside the origin would inject a second header.
pub fn origin_header(origin: &str) -> String {
    format!("Origin: {}", origin.replace(',', "%2C"))
}

pub fn mpv(settings: &PlayerSettings, req: &PlayRequest, ipc: &Path) -> Vec<OsString> {
    let mut args: Vec<OsString> = vec![
        "--force-window=immediate".into(),
        format!("--force-media-title={}", req.title).into(),
    ];
    let h = &req.headers;
    if let Some(ua) = &h.user_agent {
        args.push(format!("--user-agent={ua}").into());
    }
    if let Some(r) = &h.referrer {
        args.push(format!("--referrer={r}").into());
    }
    if let Some(o) = &h.origin {
        args.push(format!("--http-header-fields={}", origin_header(o)).into());
    }
    for (k, v) in live::file_options(settings, req.media) {
        args.push(format!("--{k}={v}").into());
    }
    if let Some(s) = req.start_s {
        args.push(format!("--start={s}").into());
    }
    if let Some(script) = live::script_path(settings) {
        let mut arg = OsString::from("--script=");
        arg.push(script);
        args.push(arg);
    }
    let mut ipc_arg = OsString::from("--input-ipc-server=");
    ipc_arg.push(ipc);
    args.push(ipc_arg);
    args.extend(
        settings
            .mpv_args
            .iter()
            .filter(|a| a.starts_with("--"))
            .map(OsString::from),
    );
    args.push("--".into());
    args.push(req.url.clone().into());
    args
}

pub fn vlc(req: &PlayRequest) -> Vec<OsString> {
    let mut args: Vec<OsString> = vec![format!("--meta-title={}", req.title).into()];
    if let Some(ua) = &req.headers.user_agent {
        args.push(format!("--http-user-agent={ua}").into());
    }
    if let Some(r) = &req.headers.referrer {
        args.push(format!("--http-referrer={r}").into());
    }
    args.push("--".into());
    args.push(req.url.clone().into());
    args
}

/// Arguments (binary excluded) from a template; tokenized before substitution so values can't add argv items.
pub fn custom(template: &str, req: &PlayRequest) -> Result<Vec<OsString>> {
    let Some(tokens) = shlex::split(template) else {
        bail!("custom player template has unbalanced quotes");
    };
    if tokens.is_empty() {
        bail!("custom player template is empty");
    }
    Ok(tokens
        .iter()
        .skip(1)
        .map(|t| substitute(t, req).into())
        .collect())
}

/// Single pass, so a value containing `{url}` is not expanded again.
fn substitute(token: &str, req: &PlayRequest) -> String {
    let h = &req.headers;
    let mut out = String::new();
    let mut rest = token;
    while let Some(start) = rest.find('{') {
        out.push_str(&rest[..start]);
        rest = &rest[start..];
        let Some((name, value)) = [
            ("{url}", Some(req.url.as_str())),
            ("{title}", Some(req.title.as_str())),
            ("{user_agent}", h.user_agent.as_deref()),
            ("{referrer}", h.referrer.as_deref()),
        ]
        .into_iter()
        .find(|(name, _)| rest.starts_with(name)) else {
            out.push('{');
            rest = &rest[1..];
            continue;
        };
        out.push_str(value.unwrap_or(""));
        rest = &rest[name.len()..];
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Headers, MediaKind};

    fn req(title: &str) -> PlayRequest {
        PlayRequest {
            url: "http://h/s.ts".into(),
            title: title.into(),
            headers: Headers::default(),
            ..Default::default()
        }
    }

    fn strs(v: Vec<OsString>) -> Vec<String> {
        v.into_iter()
            .map(|s| s.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn mpv_full_argv() {
        let mut r = req("News");
        r.media = MediaKind::Movie;
        r.headers = Headers {
            user_agent: Some("UA".into()),
            referrer: Some("http://ref".into()),
            origin: Some("http://o".into()),
        };
        let s = PlayerSettings {
            mpv_args: vec!["--fs".into(), "-bad".into(), "no".into()],
            ..Default::default()
        };
        assert_eq!(
            strs(mpv(&s, &r, Path::new("/tmp/sock"))),
            [
                "--force-window=immediate",
                "--force-media-title=News",
                "--user-agent=UA",
                "--referrer=http://ref",
                "--http-header-fields=Origin: http://o",
                "--input-ipc-server=/tmp/sock",
                "--fs",
                "--",
                "http://h/s.ts"
            ]
        );
    }

    #[test]
    fn mpv_live_adds_low_latency_vod_does_not() {
        let s = PlayerSettings::default();
        let live = strs(mpv(&s, &req("L"), Path::new("/s")));
        assert!(live.contains(&"--cache-pause=no".to_string()));
        assert!(
            live.contains(&"--demuxer-lavf-o-add=fflags=+nobuffer,live_start_index=-1".to_string())
        );
        let mut vod = req("V");
        vod.media = MediaKind::Movie;
        vod.start_s = Some(12.5);
        let vod = strs(mpv(&s, &vod, Path::new("/s")));
        assert!(!vod.contains(&"--cache-pause=no".to_string()));
        assert!(vod.contains(&"--start=12.5".to_string()));
    }

    #[test]
    fn mpv_minimal_ends_with_separator_and_url() {
        let a = strs(mpv(&PlayerSettings::default(), &req("t"), Path::new("/s")));
        assert_eq!(&a[a.len() - 2..], ["--", "http://h/s.ts"]);
        assert!(!a.iter().any(|x| x.starts_with("--user-agent")));
    }

    #[test]
    fn origin_comma_is_escaped() {
        assert_eq!(origin_header("http://a,b"), "Origin: http://a%2Cb");
    }

    #[test]
    fn vlc_argv() {
        let mut r = req("T");
        r.headers.user_agent = Some("UA".into());
        assert_eq!(
            strs(vlc(&r)),
            [
                "--meta-title=T",
                "--http-user-agent=UA",
                "--",
                "http://h/s.ts"
            ]
        );
    }

    #[test]
    fn custom_substitutes_per_token() {
        let a = custom("potplayer {url} /title={title}", &req("Hello World")).unwrap();
        assert_eq!(strs(a), ["http://h/s.ts", "/title=Hello World"]);
    }

    #[test]
    fn custom_title_cannot_add_argv_items() {
        let a = custom("player {url} {title}", &req("x\" --foo 'y")).unwrap();
        assert_eq!(strs(a), ["http://h/s.ts", "x\" --foo 'y"]);
    }

    #[test]
    fn custom_does_not_reexpand_and_handles_missing_values() {
        let a = custom("p {title}|{user_agent}|{nope}", &req("{url}")).unwrap();
        assert_eq!(strs(a), ["{url}||{nope}"]);
    }

    #[test]
    fn custom_rejects_bad_templates() {
        assert!(custom("", &req("t")).is_err());
        assert!(custom("   ", &req("t")).is_err());
        assert!(custom("p \"unclosed", &req("t")).is_err());
    }
}
