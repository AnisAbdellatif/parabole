//! External player launching (mpv, VLC, custom template) and the embedded libmpv player.

mod argv;
#[cfg(feature = "embedded-mpv")]
pub mod embed;
#[cfg(not(feature = "embedded-mpv"))]
#[path = "embed_off.rs"]
pub mod embed;
pub mod live;
mod locate;
mod mpv_ipc;
mod process;
pub mod retry;
mod settings;
mod state;
mod validate;

pub use locate::find_binary;
pub use settings::PlayerSettings;
pub use state::{Notice, PlayerState, TrackInfo, TrackKind};

use crate::model::PlayRequest;
use anyhow::{Result, anyhow, bail};
use serde::{Deserialize, Serialize};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Child;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerKind {
    /// In-app libmpv player; needs the `embedded-mpv` cargo feature.
    Embedded,
    Mpv,
    Vlc,
    Custom,
}

impl PlayerKind {
    /// Kinds offered in the settings UI.
    pub fn selectable() -> Vec<PlayerKind> {
        let mut kinds = vec![PlayerKind::Mpv, PlayerKind::Vlc, PlayerKind::Custom];
        if cfg!(feature = "embedded-mpv") {
            kinds.insert(0, PlayerKind::Embedded);
        }
        kinds
    }
}

struct Running {
    child: Child,
    kind: PlayerKind,
    ipc: Option<PathBuf>,
    /// Last `loadfile` payload, re-sent by `go_live`.
    last_payload: String,
}

/// Owns the currently running external player process (one stream at a time).
#[derive(Default)]
pub struct ExternalPlayer {
    running: Option<Running>,
}

impl ExternalPlayer {
    /// Validates the URL, stops/reuses the previous player, launches the new one.
    pub fn play(
        &mut self,
        kind: &PlayerKind,
        settings: &PlayerSettings,
        req: &PlayRequest,
    ) -> Result<()> {
        let req = validate::clean_request(req)?;
        if *kind == PlayerKind::Embedded {
            bail!("the embedded player is not an external process");
        }
        let binary = find_binary(kind, settings)
            .ok_or_else(|| anyhow!("{kind:?} player not found; set its path in settings"))?;
        if *kind == PlayerKind::Mpv && self.reuse_mpv(settings, &req) {
            return Ok(());
        }
        let (args, ipc) = match kind {
            PlayerKind::Mpv => {
                let sock = mpv_ipc::socket_path();
                (argv::mpv(settings, &req, &sock), Some(sock))
            }
            PlayerKind::Vlc => (argv::vlc(&req), None),
            PlayerKind::Custom => (argv::custom(&settings.custom_template, &req)?, None),
            PlayerKind::Embedded => unreachable!("rejected above"),
        };
        self.start(kind, &binary, &args, ipc)?;
        if let Some(r) = self.running.as_mut() {
            r.last_payload = mpv_ipc::loadfile_payload(settings, &req);
        }
        Ok(())
    }

    /// Reloads the current stream through mpv IPC, which lands on the live edge.
    pub fn go_live(&mut self) -> Result<()> {
        let Some(r) = self.running.as_ref().filter(|r| r.kind == PlayerKind::Mpv) else {
            bail!("go live needs a running mpv");
        };
        let Some(ipc) = r.ipc.clone() else {
            bail!("mpv has no IPC socket");
        };
        mpv_ipc::send(&ipc, &r.last_payload)
    }

    pub fn stop(&mut self) {
        if let Some(mut r) = self.running.take() {
            process::terminate(&mut r.child);
        }
    }

    pub fn is_running(&mut self) -> bool {
        let alive = matches!(
            self.running.as_mut().map(|r| r.child.try_wait()),
            Some(Ok(None))
        );
        if !alive {
            self.stop();
        }
        alive
    }

    fn reuse_mpv(&mut self, settings: &PlayerSettings, req: &PlayRequest) -> bool {
        let Some(ipc) = self
            .running
            .as_ref()
            .filter(|r| r.kind == PlayerKind::Mpv)
            .and_then(|r| r.ipc.clone())
        else {
            return false;
        };
        let payload = mpv_ipc::loadfile_payload(settings, req);
        let sent = self.is_running() && mpv_ipc::send(&ipc, &payload).is_ok();
        if sent && let Some(r) = self.running.as_mut() {
            r.last_payload = payload;
        }
        sent
    }

    fn start(
        &mut self,
        kind: &PlayerKind,
        binary: &Path,
        args: &[OsString],
        ipc: Option<PathBuf>,
    ) -> Result<()> {
        self.stop();
        if let Some(p) = &ipc {
            mpv_ipc::remove_stale(p);
        }
        let child = process::spawn(binary, args)?;
        self.running = Some(Running {
            child,
            kind: kind.clone(),
            ipc,
            last_payload: String::new(),
        });
        Ok(())
    }
}

impl Drop for ExternalPlayer {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::model::Headers;

    fn sleep_args() -> Vec<OsString> {
        vec!["30".into()]
    }

    #[test]
    fn start_stop_and_is_running() {
        let mut p = ExternalPlayer::default();
        assert!(!p.is_running());
        p.start(&PlayerKind::Vlc, Path::new("sleep"), &sleep_args(), None)
            .unwrap();
        assert!(p.is_running());
        p.stop();
        assert!(!p.is_running());
    }

    #[test]
    fn exited_child_is_not_running() {
        let mut p = ExternalPlayer::default();
        p.start(&PlayerKind::Vlc, Path::new("true"), &[], None)
            .unwrap();
        for _ in 0..100 {
            if !p.is_running() {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        panic!("child never reported exit");
    }

    #[test]
    fn start_replaces_previous_child() {
        let mut p = ExternalPlayer::default();
        p.start(&PlayerKind::Vlc, Path::new("sleep"), &sleep_args(), None)
            .unwrap();
        let first = p.running.as_ref().unwrap().child.id();
        p.start(&PlayerKind::Vlc, Path::new("sleep"), &sleep_args(), None)
            .unwrap();
        assert_ne!(first, p.running.as_ref().unwrap().child.id());
        assert!(p.is_running());
    }

    #[test]
    fn spawn_failure_is_reported_without_args() {
        let mut p = ExternalPlayer::default();
        let err = p
            .start(
                &PlayerKind::Vlc,
                Path::new("/no/such/binary"),
                &["http://u:secret@h".into()],
                None,
            )
            .unwrap_err();
        assert!(!format!("{err:#}").contains("secret"));
        assert!(!p.is_running());
    }

    #[test]
    fn play_rejects_bad_url_before_anything_else() {
        let mut p = ExternalPlayer::default();
        let req = PlayRequest {
            url: "--script=/tmp/x.lua".into(),
            title: String::new(),
            headers: Headers::default(),
            ..Default::default()
        };
        assert!(
            p.play(&PlayerKind::Mpv, &PlayerSettings::default(), &req)
                .is_err()
        );
    }

    #[test]
    fn mpv_reuse_requires_live_mpv_child() {
        let mut p = ExternalPlayer::default();
        let req = PlayRequest {
            url: "http://h/s".into(),
            title: "t".into(),
            headers: Headers::default(),
            ..Default::default()
        };
        assert!(!p.reuse_mpv(&PlayerSettings::default(), &req));
        p.start(&PlayerKind::Vlc, Path::new("sleep"), &sleep_args(), None)
            .unwrap();
        assert!(!p.reuse_mpv(&PlayerSettings::default(), &req));
    }
}
