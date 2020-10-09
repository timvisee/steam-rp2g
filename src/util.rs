use std::io;
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
