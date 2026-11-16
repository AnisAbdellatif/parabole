use super::autonext::AutoNext;
use super::scrobble::Scrobble;
use super::track_memory::TrackMemory;
use crate::model::{MediaKind, PlayRequest};
use crate::player::retry;
use crate::worker::{EmbedPlay, SeriesCtx, Track};
use std::time::{Duration, Instant};

pub const SAVE_EVERY: Duration = Duration::from_secs(10);
/// Playing this long resets the live retry budget.
pub const STABLE_AFTER_S: f64 = 10.0;

/// What to do when a file ends on its own.
#[derive(Debug, PartialEq, Eq)]
pub enum EndAction {
    Retry(Duration),
    GiveUp,
    Finish,
}

/// Live streams that drop (error or end) are retried with backoff; VOD just finishes.
pub fn end_action(media: MediaKind, attempt: u32, failed: bool) -> EndAction {
    match (media, failed) {
        (MediaKind::Live, _) => retry::backoff(attempt).map_or(EndAction::GiveUp, EndAction::Retry),
        (_, true) => EndAction::GiveUp,
        (_, false) => EndAction::Finish,
    }
}

/// The stream currently loaded in the embedded player.
pub struct Session {
    pub req: PlayRequest,
    pub track: Option<Track>,
    pub stream_id: Option<i64>,
    pub attempt: u32,
    pub retry_at: Option<Instant>,
    pub last_save: Instant,
    pub error: Option<String>,
    /// Last seen (position, duration) in seconds; survives the properties vanishing at end of file.
    pub known: Option<(f64, f64)>,
    pub series: Option<SeriesCtx>,
    pub autonext: Option<AutoNext>,
    pub memory: TrackMemory,
    pub opened_at: Instant,
    /// Time to the first decoded position; measured once per session.
    pub startup_ms: Option<i64>,
    pub quality_saved: bool,
    pub scrobble: Scrobble,
}

impl Session {
    pub fn new(play: EmbedPlay) -> Self {
        Self {
            req: play.req,
            track: play.track,
            stream_id: play.stream_id,
            attempt: 0,
            retry_at: None,
            last_save: Instant::now(),
            error: None,
            known: None,
            series: play.series,
            autonext: None,
            memory: TrackMemory::default(),
            opened_at: Instant::now(),
            startup_ms: None,
            quality_saved: false,
            scrobble: Scrobble::default(),
        }
    }

    pub fn is_live(&self) -> bool {
        self.req.media == MediaKind::Live
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn live_retries_three_times_then_gives_up() {
        let a = |n| end_action(MediaKind::Live, n, true);
        assert_eq!(a(0), EndAction::Retry(Duration::from_secs(1)));
        assert_eq!(a(2), EndAction::Retry(Duration::from_secs(4)));
        assert_eq!(a(3), EndAction::GiveUp);
        assert_eq!(
            end_action(MediaKind::Live, 0, false),
            EndAction::Retry(Duration::from_secs(1))
        );
    }

    #[test]
    fn vod_finishes_or_fails_without_retry() {
        assert_eq!(end_action(MediaKind::Movie, 0, false), EndAction::Finish);
        assert_eq!(end_action(MediaKind::Series, 0, true), EndAction::GiveUp);
    }
}
