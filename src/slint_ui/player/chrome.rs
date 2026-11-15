//! Chrome data read straight from mpv: the tech line and the live offset.
use crate::player::embed::MpvCore;

fn channels_label(n: i64) -> String {
    match n {
        1 => "Mono".into(),
        2 => "Stereo".into(),
        6 => "5.1".into(),
        8 => "7.1".into(),
        n => format!("{n}ch"),
    }
}

/// e.g. "1080p · 5.1"; parts mpv doesn't know yet are left out.
pub fn tech_line(core: &MpvCore) -> String {
    let mpv = core.mpv();
    let height = mpv
        .get_property::<i64>("video-params/h")
        .ok()
        .filter(|h| *h > 0);
    let channels = mpv
        .get_property::<i64>("audio-params/channel-count")
        .ok()
        .filter(|c| *c > 0);
    [
        height.map(|h| format!("{h}p")),
        channels.map(channels_label),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(" · ")
}

/// Seconds buffered ahead of the playhead: how far a live stream trails its edge.
/// Reuses the demuxer cache reading of the stats overlay.
pub fn behind_s(core: &MpvCore) -> f32 {
    core.mpv()
        .get_property::<f64>("demuxer-cache-duration")
        .map_or(0.0, |s| s.max(0.0) as f32)
}

#[cfg(test)]
mod tests {
    use super::channels_label;

    #[test]
    fn channel_labels() {
        assert_eq!(channels_label(2), "Stereo");
        assert_eq!(channels_label(6), "5.1");
        assert_eq!(channels_label(3), "3ch");
    }
}
