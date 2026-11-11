//! "Stats for nerds": playback numbers read from the embedded mpv, and their display lines.

/// `None` means mpv doesn't know the value (yet), e.g. no audio track or still opening.
#[derive(Clone, Debug, Default)]
pub struct Stats {
    pub video_codec: Option<String>,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub fps: Option<f64>,
    pub pixel_format: Option<String>,
    pub hwdec: Option<String>,
    pub video_bitrate: Option<f64>,
    pub audio_codec: Option<String>,
    pub sample_rate: Option<i64>,
    pub channels: Option<i64>,
    pub audio_bitrate: Option<f64>,
    pub container: Option<String>,
    pub buffer_s: Option<f64>,
    pub net_bytes_s: Option<i64>,
    pub dropped_output: Option<i64>,
    pub dropped_decoder: Option<i64>,
    pub av_sync_s: Option<f64>,
}

const NONE: &str = "–";

impl Stats {
    /// Label/value rows in display order; unknown parts show as "–".
    pub fn lines(&self) -> Vec<(&'static str, String)> {
        let size = match (self.width, self.height) {
            (Some(w), Some(h)) => format!("{w}×{h}"),
            _ => NONE.into(),
        };
        let hwdec = match self.hwdec.as_deref() {
            None | Some("") | Some("no") => "software".to_string(),
            Some(h) => h.to_string(),
        };
        vec![
            (
                "Video",
                join(&[
                    text(&self.video_codec),
                    size,
                    num(self.fps, |f| format!("{f:.2} fps")),
                ]),
            ),
            ("Decoder", join(&[text(&self.pixel_format), hwdec])),
            ("Video bitrate", num(self.video_bitrate, bitrate)),
            (
                "Audio",
                join(&[
                    text(&self.audio_codec),
                    num(self.sample_rate, |r| {
                        format!("{:.1} kHz", r as f64 / 1000.0)
                    }),
                    num(self.channels, |c| format!("{c} ch")),
                ]),
            ),
            ("Audio bitrate", num(self.audio_bitrate, bitrate)),
            ("Container", text(&self.container)),
            ("Buffer", num(self.buffer_s, |s| format!("{s:.1} s"))),
            ("Network", num(self.net_bytes_s, |b| bytes_per_s(b as f64))),
            (
                "Dropped frames",
                format!(
                    "{} output, {} decoder",
                    num(self.dropped_output, |n| n.to_string()),
                    num(self.dropped_decoder, |n| n.to_string())
                ),
            ),
            (
                "A/V sync",
                num(self.av_sync_s, |s| format!("{:+.0} ms", s * 1000.0)),
            ),
        ]
    }
}

fn text(v: &Option<String>) -> String {
    v.clone()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| NONE.into())
}

fn num<T: Copy>(v: Option<T>, f: impl Fn(T) -> String) -> String {
    v.map_or_else(|| NONE.into(), f)
}

fn join(parts: &[String]) -> String {
    parts.join(" · ")
}

fn bitrate(bits_s: f64) -> String {
    if bits_s >= 1_000_000.0 {
        format!("{:.2} Mbps", bits_s / 1_000_000.0)
    } else {
        format!("{:.0} kbps", bits_s / 1000.0)
    }
}

fn bytes_per_s(b: f64) -> String {
    if b >= 1_048_576.0 {
        format!("{:.1} MiB/s", b / 1_048_576.0)
    } else {
        format!("{:.0} KiB/s", b / 1024.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn value(s: &Stats, label: &str) -> String {
        s.lines()
            .into_iter()
            .find(|(l, _)| *l == label)
            .map(|(_, v)| v)
            .unwrap_or_default()
    }

    #[test]
    fn unknown_values_show_dashes() {
        let s = Stats::default();
        assert_eq!(value(&s, "Video"), "– · – · –");
        assert_eq!(value(&s, "Decoder"), "– · software");
        assert_eq!(value(&s, "Dropped frames"), "– output, – decoder");
    }

    #[test]
    fn known_values_are_formatted() {
        let s = Stats {
            video_codec: Some("h264".into()),
            width: Some(1920),
            height: Some(1080),
            fps: Some(25.0),
            hwdec: Some("videotoolbox".into()),
            video_bitrate: Some(4_200_000.0),
            sample_rate: Some(48000),
            channels: Some(2),
            audio_bitrate: Some(128_000.0),
            net_bytes_s: Some(3 * 1_048_576 / 2),
            av_sync_s: Some(-0.012),
            ..Default::default()
        };
        assert_eq!(value(&s, "Video"), "h264 · 1920×1080 · 25.00 fps");
        assert_eq!(value(&s, "Decoder"), "– · videotoolbox");
        assert_eq!(value(&s, "Video bitrate"), "4.20 Mbps");
        assert_eq!(value(&s, "Audio"), "– · 48.0 kHz · 2 ch");
        assert_eq!(value(&s, "Audio bitrate"), "128 kbps");
        assert_eq!(value(&s, "Network"), "1.5 MiB/s");
        assert_eq!(value(&s, "A/V sync"), "-12 ms");
    }
}
