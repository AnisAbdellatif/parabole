//! Seeds the default-path DB for the VM e2e run: one M3U file source, embedded player on.
//! Usage: `vm_seed <playlist.m3u> [mpv.log]` (run with `HOME` pointing at a scratch dir).
use anyhow::{Context, Result};
use iptv_player::db::Db;
use iptv_player::model::{Source, SourceKind};
use iptv_player::player::{PlayerKind, PlayerSettings};
use iptv_player::sources::m3u;
use iptv_player::ui::db_path;
use iptv_player::worker::player_settings;

fn main() -> Result<()> {
    let m3u_path = std::env::args()
        .nth(1)
        .context("usage: vm_seed <playlist.m3u>")?;
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
