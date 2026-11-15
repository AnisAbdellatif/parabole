//! Slint front end. Run with `cargo run --features slint-ui --bin iptv-player-slint`.
use anyhow::{Context, Result};
use slint::ComponentHandle;

slint::include_modules!();

pub fn run() -> Result<()> {
    let win = AppWindow::new().context("create window")?;
    win.run().context("run event loop")?;
    Ok(())
}
