//! "Next episode" countdown shown on the video when an episode ends by itself.
use super::session::Session;
use std::time::{Duration, Instant};

const COUNTDOWN: Duration = Duration::from_secs(5);

pub struct AutoNext {
    pub episode_id: String,
    pub label: String,
    pub deadline: Instant,
}

/// The episode after the one `s` was playing, when it came from the series view.
pub(crate) fn find_next(s: &Session) -> Option<AutoNext> {
    let ctx = s.series.as_ref()?;
    let id = s.track.as_ref()?.key.strip_prefix("ep:")?;
    let (season, ep) = crate::episodes::next(&ctx.seasons, id)?;
    Some(AutoNext {
        episode_id: ep.id.clone(),
        label: crate::episodes::label(season, ep),
        deadline: Instant::now() + COUNTDOWN,
    })
}
