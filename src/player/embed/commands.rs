use super::EmbeddedPlayer;
use crate::model::PlayRequest;
use crate::player::PlayerSettings;
use crate::player::argv::origin_header;
use crate::player::{PlayerState, TrackKind, live};
use anyhow::{Result, anyhow};

/// Per-file options of a request, as `loadfile` takes them.
fn file_options(settings: &PlayerSettings, req: &PlayRequest) -> Vec<(&'static str, String)> {
    let mut opts = vec![("force-media-title", req.title.clone())];
    let h = &req.headers;
    if let Some(ua) = &h.user_agent {
        opts.push(("user-agent", ua.clone()));
    }
    if let Some(r) = &h.referrer {
        opts.push(("referrer", r.clone()));
    }
    if let Some(o) = &h.origin {
        opts.push(("http-header-fields", origin_header(o)));
    }
    if let Some(s) = req.start_s {
        opts.push(("start", s.to_string()));
    }
    opts.extend(live::file_options(settings, req.media));
    opts
}

impl EmbeddedPlayer {
    /// Validates and loads a stream, replacing the current one.
    pub fn play(&mut self, settings: &PlayerSettings, req: &PlayRequest) -> Result<()> {
        let req = crate::player::validate::clean_request(req)?;
        let opts = live::options_string(&file_options(settings, &req));
        self.state.reset_file();
        self.mpv
            .command("loadfile", &[&req.url, "replace", "-1", &opts])
            .map_err(|e| anyhow!("loadfile failed: {e}"))?;
        self.state.paused = false;
        self.last = Some(req);
        Ok(())
    }

    /// Reloads the current stream, which lands on the live edge.
    pub fn reload(&mut self, settings: &PlayerSettings) -> Result<()> {
        let req = self
            .last
            .clone()
            .ok_or_else(|| anyhow!("nothing is playing"))?;
        self.play(settings, &req)
    }

    pub fn stop(&mut self) {
        let _ = self.mpv.command("stop", &[]);
        self.state = PlayerState {
            volume: self.state.volume,
            muted: self.state.muted,
            ..Default::default()
        };
        self.last = None;
    }

    pub fn is_loaded(&self) -> bool {
        self.last.is_some()
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.state.paused = paused;
        let _ = self.mpv.set_property("pause", paused);
    }

    pub fn set_volume(&mut self, volume: f64) {
        self.state.volume = volume;
        let _ = self.mpv.set_property("volume", volume);
    }

    pub fn set_muted(&mut self, muted: bool) {
        self.state.muted = muted;
        let _ = self.mpv.set_property("mute", muted);
    }

    pub fn seek_to(&self, seconds: f64) {
        let _ = self
            .mpv
            .command("seek", &[&seconds.to_string(), "absolute"]);
    }

    /// `None` turns the track type off.
    pub fn select_track(&self, kind: TrackKind, id: Option<i64>) {
        let prop = match kind {
            TrackKind::Audio => "aid",
            TrackKind::Sub => "sid",
            TrackKind::Video => "vid",
        };
        let _ = self
            .mpv
            .set_property(prop, id.map_or("no".to_string(), |i| i.to_string()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Headers, MediaKind};

    #[test]
    fn live_request_gets_latency_options_vod_gets_start() {
        let s = PlayerSettings::default();
        let mut req = PlayRequest {
            url: "http://h/s".into(),
            title: "T".into(),
            headers: Headers {
                origin: Some("http://o".into()),
                ..Default::default()
            },
            ..Default::default()
        };
        fn keys(o: Vec<(&'static str, String)>) -> Vec<&'static str> {
            o.into_iter().map(|(k, _)| k).collect()
        }
        let live = keys(file_options(&s, &req));
        assert!(live.contains(&"cache-pause") && live.contains(&"http-header-fields"));
        req.media = MediaKind::Movie;
        req.start_s = Some(90.0);
        let vod = keys(file_options(&s, &req));
        assert!(!vod.contains(&"cache-pause") && vod.contains(&"start"));
    }
}
