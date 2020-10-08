use std::fs;
use std::io;
use std::path::Path;
use std::process::{Command, Output};

/// Invoke system command.
///
/// Command is passed as string to `/bin/sh`.
///
/// Returns an error on failure.
pub fn invoke_cmd(cmd: &str) -> io::Result<Output> {
    Command::new("sh").arg("-c").arg(cmd).output()
}

/// Check we are on a supported platform.
pub fn is_supported_platform() -> bool {
    std::env::consts::OS == "linux"
}

/// Report if we are on an unsupported platform.
pub fn report_unsupported_platform() {
    if !is_supported_platform() {
        eprintln!("Error: unsupported platform, may not work");
    }
}

/// Remove all contents in the given directory.
pub fn remove_dir_contents(dir: &Path) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry.metadata()?.is_dir() {
            fs::remove_dir_all(entry.path())?;
        } else {
            fs::remove_file(entry.path())?;
        }
    }
    Ok(())
}
