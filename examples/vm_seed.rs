//! Seeds the default-path DB for the VM e2e run: one M3U file source, embedded player on.
//! Usage (run with `HOME` pointing at a scratch dir):
//!   `vm_seed <playlist.m3u> [mpv.log]`  local M3U file source, verbose mpv log
//!   `vm_seed --real <url-file>`         Xtream source from a file holding one URL; the app's first refresh fetches it
//!   `vm_seed --wait <secs>`             wait for that refresh; prints only "refresh ok" or "refresh failed"
//! Real mode never prints the file content or error texts (they can contain the credentials).
use anyhow::{Context, Result, bail};
use parabole::db::Db;
use parabole::model::{Source, SourceInput, SourceKind};
use parabole::player::{PlayerKind, PlayerSettings};
use parabole::slint_ui::paths::db_path;
use parabole::sources::detect::detect;
use parabole::sources::m3u;
use parabole::worker::player_settings;

fn main() -> Result<()> {
    let arg1 = std::env::args()
        .nth(1)
        .context("usage: vm_seed <playlist.m3u> | --real <url-file> | --wait <secs>")?;
    match arg1.as_str() {
        "--real" => seed_real(&std::env::args().nth(2).context("--real needs a file")?),
        "--wait" => wait_refresh(
            std::env::args()
                .nth(2)
                .and_then(|s| s.parse().ok())
                .unwrap_or(300),
        ),
        _ => seed_local(&arg1),
    }
}

fn seed_real(path: &str) -> Result<()> {
    let text = std::fs::read_to_string(path).context("read url file")?;
    let Ok(SourceInput::Xtream {
        base,
        username,
        password,
    }) = detect(&text)
    else {
        bail!("url file is not a recognised Xtream URL");
    };
    let db = Db::open(&db_path()?)?;
    let profile = db.ensure_default_profile()?;
    // auto_refresh_hours stays at its default: the app's startup AutoRefresh fetches the new (never refreshed) source
    player_settings::save(
        &db,
        &PlayerSettings {
            default: PlayerKind::Embedded,
            // no log file or verbose level: mpv's verbose log prints stream URLs
            mpv_args: [
                "--ao=null",
                "--scale=bilinear",
                "--cscale=bilinear",
                "--dither=no",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            ..PlayerSettings::default()
        },
    )?;
    db.add_source(
        profile,
        &Source {
            id: 0,
            name: "Real".into(),
            kind: SourceKind::Xtream,
            url: base,
            mirrors: Vec::new(),
            username: Some(username),
            password: Some(password),
            user_agent: None,
            output_format: None,
            last_refresh: None,
            last_error: None,
        },
    )?;
    Ok(())
}

fn wait_refresh(secs: u64) -> Result<()> {
    let end = std::time::Instant::now() + std::time::Duration::from_secs(secs);
    while std::time::Instant::now() < end {
        // the app holds the DB too: a transient open/lock error just means "ask again"
        if let Ok(db) = Db::open(&db_path()?)
            && let Ok(profile) = db.ensure_default_profile()
            && let Ok(sources) = db.profile_sources(profile)
            && let Some(s) = sources.first()
        {
            if s.last_error.is_some() {
                println!("refresh failed");
                std::process::exit(1);
            }
            if s.last_refresh.is_some() {
                println!("refresh ok");
                return Ok(());
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
    println!("refresh timed out");
    std::process::exit(1)
}

fn seed_local(m3u_path: &str) -> Result<()> {
    let m3u_path = m3u_path.to_string();
    let mut db = Db::open(&db_path()?)?;
    let profile = db.ensure_default_profile()?;
    db.set_setting(None, "auto_refresh_hours", "0")?;
    player_settings::save(
        &db,
        &PlayerSettings {
            default: PlayerKind::Embedded,
            // Xvfb guest: no audio sink; llvmpipe renders mpv's default scaler (and dither) as black lines or black
            // frames (stock `mpv --vo=gpu` does too), so use bilinear. The log also proves stored mpv_args apply.
            mpv_args: [
                "--ao=null",
                "--scale=bilinear",
                "--cscale=bilinear",
                "--dither=no",
            ]
            .into_iter()
            .map(String::from)
            .chain(
                std::env::args()
                    .nth(2)
                    .into_iter()
                    .flat_map(|log| [format!("--log-file={log}"), "--msg-level=all=v".to_string()]),
            )
            .collect(),
            ..PlayerSettings::default()
        },
    )?;
    let source = db.add_source(
        profile,
        &Source {
            id: 0,
            name: "E2E".into(),
            kind: SourceKind::M3uFile,
            url: m3u_path.clone(),
            mirrors: Vec::new(),
            username: None,
            password: None,
            user_agent: None,
            output_format: None,
            last_refresh: None,
            last_error: None,
        },
    )?;
    let data = std::fs::read(&m3u_path).context("read playlist")?;
    db.replace_source_data(source, &m3u::parse(&*data)?)?;
    Ok(())
}
