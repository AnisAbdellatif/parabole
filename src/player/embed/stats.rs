use super::EmbeddedPlayer;
use crate::player::Stats;
use libmpv2::{GetData, Mpv};

fn get<T: GetData>(mpv: &Mpv, name: &str) -> Option<T> {
    mpv.get_property(name).ok()
}

impl EmbeddedPlayer {
    /// Reads the numbers on demand; only called while the overlay is shown.
    pub fn stats(&self) -> Stats {
        let m = &*self.mpv;
        Stats {
            video_codec: get(m, "video-format"),
            width: get(m, "video-params/w"),
            height: get(m, "video-params/h"),
            fps: get(m, "estimated-vf-fps").or_else(|| get(m, "container-fps")),
            pixel_format: get(m, "video-params/pixelformat"),
            hwdec: get(m, "hwdec-current"),
            video_bitrate: get(m, "video-bitrate"),
            audio_codec: get(m, "audio-codec-name"),
            sample_rate: get(m, "audio-params/samplerate"),
            channels: get(m, "audio-params/channel-count"),
            audio_bitrate: get(m, "audio-bitrate"),
            container: get(m, "file-format"),
            buffer_s: get(m, "demuxer-cache-duration"),
            net_bytes_s: get(m, "cache-speed"),
            dropped_output: get(m, "frame-drop-count"),
            dropped_decoder: get(m, "decoder-frame-drop-count"),
            av_sync_s: get(m, "avsync"),
        }
    }
}
