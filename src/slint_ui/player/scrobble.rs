//! Reports embedded playback to Trakt: start, pause/resume and stop with the watched share.
use super::session::Session;
use crate::model::MediaKind;
use crate::trakt::scrobble::Action;
use crate::worker::{Command, Handle};

#[derive(Default)]
pub struct Scrobble {
    started: bool,
    paused: bool,
    /// Stopped for good (file finished); a lingering session must not start again.
    ended: bool,
}

/// What to tell Trakt now, given the last known (position, duration) and the paused state.
fn next(state: &Scrobble, known: Option<(f64, f64)>, paused: bool) -> Option<Action> {
    if state.ended || known.is_none_or(|(_, dur)| dur <= 0.0) {
        return None;
    }
    if !state.started {
        return (!paused).then_some(Action::Start);
    }
    (paused != state.paused).then_some(if paused { Action::Pause } else { Action::Start })
}

fn percent(known: Option<(f64, f64)>) -> f32 {
    known.map_or(0.0, |(pos, dur)| {
        (pos / dur * 100.0).clamp(0.0, 100.0) as f32
    })
}

fn command(s: &Session, action: Action, progress: f32) -> Option<Command> {
    let track = s.track.clone()?;
    let (series_key, episode) = match track.media {
        MediaKind::Live => return None,
        MediaKind::Movie => (None, None),
        MediaKind::Series => {
            let ctx = s.series.as_ref()?;
            let id = track.key.strip_prefix("ep:")?;
            let (season, ep) = crate::episodes::find(&ctx.seasons, id)?;
            (
                Some(ctx.key.clone()),
                Some((season.number, ep.episode_num?)),
            )
        }
    };
    Some(Command::Scrobble {
        track,
        series_key,
        episode,
        action,
        progress,
    })
}

fn send(worker: &Handle, s: &Session, action: Action, progress: f32) {
    if let Some(cmd) = command(s, action, progress) {
        worker.send(cmd);
    }
}

pub fn tick(worker: &Handle, s: &mut Session, paused: bool) {
    if let Some(action) = next(&s.scrobble, s.known, paused) {
        send(worker, s, action, percent(s.known));
        s.scrobble.started = true;
    }
    s.scrobble.paused = paused;
}

pub fn stop(worker: &Handle, s: &mut Session, finished: bool) {
    if s.scrobble.started {
        let progress = if finished { 100.0 } else { percent(s.known) };
        send(worker, s, Action::Stop, progress);
    }
    s.scrobble.started = false;
    s.scrobble.ended = true;
}

#[cfg(test)]
mod tests {
    use super::*;

    const KNOWN: Option<(f64, f64)> = Some((30.0, 120.0));

    fn state(started: bool, paused: bool) -> Scrobble {
        Scrobble {
            started,
            paused,
            ended: false,
        }
    }

    #[test]
    fn waits_for_a_known_duration() {
        assert_eq!(next(&state(false, false), None, false), None);
        assert_eq!(next(&state(false, false), Some((0.0, 0.0)), false), None);
    }

    #[test]
    fn starts_once_playing() {
        assert_eq!(
            next(&state(false, false), KNOWN, false),
            Some(Action::Start)
        );
        assert_eq!(next(&state(false, false), KNOWN, true), None);
    }

    #[test]
    fn follows_pause_and_resume_only_on_change() {
        assert_eq!(next(&state(true, false), KNOWN, false), None);
        assert_eq!(next(&state(true, false), KNOWN, true), Some(Action::Pause));
        assert_eq!(next(&state(true, true), KNOWN, true), None);
        assert_eq!(next(&state(true, true), KNOWN, false), Some(Action::Start));
    }

    #[test]
    fn never_restarts_after_the_end() {
        let ended = Scrobble {
            ended: true,
            ..Scrobble::default()
        };
        assert_eq!(next(&ended, KNOWN, false), None);
    }

    #[test]
    fn percent_is_clamped() {
        assert_eq!(percent(KNOWN), 25.0);
        assert_eq!(percent(Some((130.0, 120.0))), 100.0);
        assert_eq!(percent(None), 0.0);
    }
}
