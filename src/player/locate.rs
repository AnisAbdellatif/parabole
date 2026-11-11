use crate::player::{PlayerKind, PlayerSettings};
use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// Locate a player binary: settings override, PATH, then platform default locations.
pub fn find_binary(kind: &PlayerKind, settings: &PlayerSettings) -> Option<PathBuf> {
    match kind {
        PlayerKind::Mpv => locate(settings.mpv_path.as_deref(), "mpv", mpv_defaults()),
        PlayerKind::Vlc => locate(settings.vlc_path.as_deref(), "vlc", vlc_defaults()),
        PlayerKind::Custom => {
            let first = shlex::split(&settings.custom_template)?
                .into_iter()
                .next()?;
            if first.contains(['/', '\\']) {
                let p = PathBuf::from(first);
                return p.is_file().then_some(p);
            }
            search_path(&first, &env::var_os("PATH")?)
        }
    }
}

fn locate(over: Option<&Path>, name: &str, defaults: Vec<PathBuf>) -> Option<PathBuf> {
    if let Some(p) = over.filter(|p| p.is_file()) {
        return Some(p.to_path_buf());
    }
    env::var_os("PATH")
        .and_then(|path| search_path(name, &path))
        .or_else(|| defaults.into_iter().find(|p| p.is_file()))
}

fn search_path(name: &str, path_var: &OsStr) -> Option<PathBuf> {
    let file = if cfg!(windows) && !name.ends_with(".exe") {
        format!("{name}.exe")
    } else {
        name.to_string()
    };
    env::split_paths(path_var)
        .map(|dir| dir.join(&file))
        .find(|p| p.is_file())
}

fn env_dir(var: &str, rel: &str) -> Option<PathBuf> {
    env::var_os(var).map(|v| PathBuf::from(v).join(rel))
}

fn mpv_defaults() -> Vec<PathBuf> {
    if cfg!(target_os = "macos") {
        [
            "/opt/homebrew/bin/mpv",
            "/usr/local/bin/mpv",
            "/Applications/mpv.app/Contents/MacOS/mpv",
        ]
        .map(PathBuf::from)
        .to_vec()
    } else if cfg!(windows) {
        [
            env_dir("ProgramFiles", r"mpv\mpv.exe"),
            env_dir("ProgramFiles(x86)", r"mpv\mpv.exe"),
            env_dir("USERPROFILE", r"scoop\shims\mpv.exe"),
        ]
        .into_iter()
        .flatten()
        .collect()
    } else {
        ["/usr/bin/mpv", "/snap/bin/mpv"]
            .map(PathBuf::from)
            .to_vec()
    }
}

fn vlc_defaults() -> Vec<PathBuf> {
    if cfg!(target_os = "macos") {
        vec![PathBuf::from("/Applications/VLC.app/Contents/MacOS/VLC")]
    } else if cfg!(windows) {
        [
            env_dir("ProgramFiles", r"VideoLAN\VLC\vlc.exe"),
            env_dir("ProgramFiles(x86)", r"VideoLAN\VLC\vlc.exe"),
            env_dir("USERPROFILE", r"scoop\shims\vlc.exe"),
        ]
        .into_iter()
        .flatten()
        .collect()
    } else {
        ["/usr/bin/vlc", "/snap/bin/vlc"]
            .map(PathBuf::from)
            .to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn scratch(tag: &str) -> PathBuf {
        let d = env::temp_dir().join(format!("iptv-locate-{tag}-{}", std::process::id()));
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn path_walk_finds_binary() {
        let d = scratch("walk");
        let name = if cfg!(windows) {
            "fakeplayer.exe"
        } else {
            "fakeplayer"
        };
        fs::write(d.join(name), "").unwrap();
        let path = env::join_paths([PathBuf::from("/nonexistent-dir"), d.clone()]).unwrap();
        assert_eq!(search_path("fakeplayer", &path), Some(d.join(name)));
        assert_eq!(search_path("missing-player", &path), None);
        fs::remove_dir_all(d).unwrap();
    }

    #[test]
    fn settings_override_wins_only_if_it_exists() {
        let d = scratch("over");
        let bin = d.join("mympv");
        fs::write(&bin, "").unwrap();
        let s = PlayerSettings {
            mpv_path: Some(bin.clone()),
            ..Default::default()
        };
        assert_eq!(find_binary(&PlayerKind::Mpv, &s), Some(bin));
        assert_eq!(
            locate(Some(&d.join("nope")), "definitely-not-a-player", vec![]),
            None
        );
        fs::remove_dir_all(d).unwrap();
    }

    #[test]
    fn custom_resolves_first_token() {
        let d = scratch("custom");
        let bin = d.join("cp");
        fs::write(&bin, "").unwrap();
        let s = PlayerSettings {
            custom_template: format!("'{}' {{url}}", bin.display()),
            ..Default::default()
        };
        assert_eq!(find_binary(&PlayerKind::Custom, &s), Some(bin));
        assert_eq!(
            find_binary(&PlayerKind::Custom, &PlayerSettings::default()),
            None
        );
        fs::remove_dir_all(d).unwrap();
    }
}
