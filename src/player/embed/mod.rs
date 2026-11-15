//! In-app player: libmpv with the OpenGL render API, drawn into the Slint window.
//! libmpv and its GL state live on the UI thread; the worker never touches this.

mod commands;
mod core;
mod events;
pub(crate) mod render;
mod stats;

pub use self::core::MpvCore;

/// Options set once at init; headers and low-latency options are per file (see `commands`).
const INIT_OPTIONS: [(&str, &str); 7] = [
    ("vo", "libmpv"),
    ("hwdec", "auto-safe"),
    ("ytdl", "no"),
    ("input-default-bindings", "no"),
    ("osc", "no"),
    ("keep-open", "no"),
    (
        "stream-lavf-o",
        "reconnect=1,reconnect_streamed=1,reconnect_delay_max=5",
    ),
];

/// `--key=value` or `--flag` (= `yes`); other shapes are ignored.
fn split_mpv_arg(arg: &str) -> Option<(&str, &str)> {
    let body = arg.strip_prefix("--")?;
    Some(body.split_once('=').unwrap_or((body, "yes")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mpv_arg_splitting() {
        assert_eq!(split_mpv_arg("--volume=50"), Some(("volume", "50")));
        assert_eq!(split_mpv_arg("--mute"), Some(("mute", "yes")));
        assert_eq!(split_mpv_arg("-x"), None);
    }
}
