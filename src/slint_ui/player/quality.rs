//! Records the measured quality of live streams played in the embedded player.
use super::session::{STABLE_AFTER_S, Session};
use crate::db::Db;
use crate::model::{MediaKind, Quality};
use crate::player::embed::MpvCore;
use crate::worker::Track;

fn live_track(s: &Session) -> Option<&Track> {
    s.track.as_ref().filter(|t| t.media == MediaKind::Live)
}

/// Notes startup time, then saves one measurement once playback is stable.
pub fn on_tick(db: &Db, core: &MpvCore, s: &mut Session) {
    let Some(pos) = core.state().time_pos else {
        return;
    };
    if s.startup_ms.is_none() {
        s.startup_ms = Some(s.opened_at.elapsed().as_millis() as i64);
    }
    if !s.is_live() || pos <= STABLE_AFTER_S || s.quality_saved {
        return;
    }
    s.quality_saved = true;
    let stats = core.stats();
    let (Some(track), Some(_)) = (live_track(s), stats.height) else {
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
    let _ = db.save_quality(track.source, &track.key, &q);
}

pub fn on_give_up(db: &Db, s: &Session) {
    if let Some(track) = live_track(s) {
        let _ = db.record_failure(track.source, &track.key);
    }
}
