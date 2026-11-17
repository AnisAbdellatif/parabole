//! Records the measured quality of live streams played in the embedded player.
use super::session::{STABLE_AFTER_S, Session};
use crate::model::{MediaKind, Quality};
use crate::player::embed::MpvCore;
use crate::slint_ui::writer::Writer;
use crate::worker::Track;

fn live_track(s: &Session) -> Option<&Track> {
    s.track.as_ref().filter(|t| t.media == MediaKind::Live)
}

/// Notes startup time, then saves one measurement once playback is stable.
pub fn on_tick(writer: &Writer, core: &MpvCore, s: &mut Session) {
    if core.state().time_pos.is_none() {
        return;
    }
    let elapsed_ms = s.opened_at.elapsed().as_millis() as i64;
    let startup_ms = *s.startup_ms.get_or_insert(elapsed_ms);
    // Live time_pos can start at the stream's own timestamp, so count wall time since the first frame.
    let stable = (elapsed_ms - startup_ms) as f64 > STABLE_AFTER_S * 1000.0;
    if !s.is_live() || !stable || s.quality_saved {
        return;
    }
    let stats = core.stats();
    if stats.height.is_none() {
        return;
    }
    s.quality_saved = true;
    let Some(track) = live_track(s) else {
        return;
    };
    let q = Quality {
        width: stats.width,
        height: stats.height,
        fps: stats.fps,
        video_codec: stats.video_codec,
        bitrate: stats.video_bitrate,
        startup_ms: s.startup_ms,
        fails: 0,
    };
    // Like track_memory: a failed write only loses a hint, never worth a toast.
    let (source, key) = (track.source, track.key.clone());
    writer.run(move |db| {
        let _ = db.save_quality(source, &key, &q);
    });
}

pub fn on_give_up(writer: &Writer, s: &Session) {
    if let Some(track) = live_track(s) {
        let (source, key) = (track.source, track.key.clone());
        writer.run(move |db| {
            let _ = db.record_failure(source, &key);
        });
    }
}
