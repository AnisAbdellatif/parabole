use crate::player::live;
use anyhow::{Context, Result};
use directories::ProjectDirs;
use std::path::{Path, PathBuf};

pub fn db_path() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("dev", "parabole", "Parabole").context("no home directory")?;
    std::fs::create_dir_all(dirs.data_dir()).context("create data dir")?;
    Ok(dirs.data_dir().join("iptv.db"))
}

pub fn install_mpv_script(db: &Path) {
    if let Some(dir) = db.parent() {
        let _ = live::install_script(dir);
    }
}
