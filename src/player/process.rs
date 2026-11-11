use anyhow::{Context, Result};
use std::ffi::OsString;
use std::path::Path;
use std::process::{Child, Command, Stdio};

/// Direct exec, never via a shell.
pub fn spawn(binary: &Path, args: &[OsString]) -> Result<Child> {
    Command::new(binary)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .with_context(|| format!("failed to launch {}", binary.display()))
}

/// Kill and reap so no zombie is left behind.
pub fn terminate(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}
