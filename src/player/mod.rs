//! External player launching (mpv, VLC, custom template). Owner: player agent.

mod argv;
mod locate;
mod mpv_ipc;
mod process;
mod validate;

pub use locate::find_binary;

use crate::model::PlayRequest;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Child;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerKind {
    Mpv,
    Vlc,
    Custom,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlayerSettings {
    pub default: PlayerKind,
    /// None = auto-detect.
    pub mpv_path: Option<PathBuf>,
    pub vlc_path: Option<PathBuf>,
    /// e.g. `potplayer {url} /title={title}`; placeholders: {url} {title} {user_agent} {referrer}
    pub custom_template: String,
    /// Extra `--key=value` args appended to mpv.
    pub mpv_args: Vec<String>,
}

impl Default for PlayerSettings {
    fn default() -> Self {
        Self {
            default: PlayerKind::Mpv,
            mpv_path: None,
            vlc_path: None,
            custom_template: String::new(),
            mpv_args: Vec::new(),
        }
    }
}

struct Running {
    child: Child,
    kind: PlayerKind,
    ipc: Option<PathBuf>,
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
        let binary = find_binary(kind, settings)
            .ok_or_else(|| anyhow!("{kind:?} player not found; set its path in settings"))?;
        if *kind == PlayerKind::Mpv && self.reuse_mpv(&req) {
            return Ok(());
        }
        let (args, ipc) = match kind {
            PlayerKind::Mpv => {
                let sock = mpv_ipc::socket_path();
                (argv::mpv(settings, &req, &sock), Some(sock))
            }
            PlayerKind::Vlc => (argv::vlc(&req), None),
            PlayerKind::Custom => (argv::custom(&settings.custom_template, &req)?, None),
        };
        self.start(kind, &binary, &args, ipc)
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

    fn reuse_mpv(&mut self, req: &PlayRequest) -> bool {
        let Some(ipc) = self
            .running
            .as_ref()
            .filter(|r| r.kind == PlayerKind::Mpv)
            .and_then(|r| r.ipc.clone())
        else {
            return false;
        };
        self.is_running() && mpv_ipc::send(&ipc, &mpv_ipc::loadfile_payload(req)).is_ok()
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
        };
        assert!(!p.reuse_mpv(&req));
        p.start(&PlayerKind::Vlc, Path::new("sleep"), &sleep_args(), None)
            .unwrap();
        assert!(!p.reuse_mpv(&req));
    }
}
