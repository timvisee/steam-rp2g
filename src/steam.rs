use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::util;

/// Invoke a Steam URL.
///
/// Give the path to invoke. `install/1` will invoke `steam://install/1`.
fn invoke_steam_url(path: &str) {
    util::invoke_cmd(&format!("xdg-open steam://{}", path)).expect("failed to invoke Steam URL");
}

/// Initiate a game installation through Steam.
pub fn invoke_steam_install(game_id: usize) {
    invoke_steam_url(&format!("install/{}", game_id));
}

/// Initiate running a game through Steam.
pub fn invoke_steam_run(game_id: usize) {
    invoke_steam_url(&format!("run/{}", game_id));
}

/// Find the Steam games directory.
pub fn find_steam_games_dir() -> PathBuf {
    #[allow(deprecated)]
    let home = env::home_dir().expect("unable to determine user home directory");

    let mut steam = home.clone();
    steam.push(".steam/steam/steamapps/common/");

    fs::canonicalize(steam).expect("could not find Steam games directory")
}

/// Find directories of steam games.
pub fn find_steam_game_dirs() -> Vec<PathBuf> {
    crate::fs::ls(&find_steam_games_dir())
        .expect("failed to list Steam game dirs")
        .into_iter()
        .filter(|f| f.is_dir())
        .collect()
}

/// Find game binaries.
pub fn find_game_bins(dir: &Path) -> Vec<PathBuf> {
    // TODO: only executables, filter by [.exe, .x86_64] and such
    crate::fs::ls(dir)
        .expect("failed to list Steam game dirs")
        .into_iter()
        .filter(|f| is_bin(&f))
        .collect()
}

/// Check whether given file is considered a binary.
pub fn is_bin(path: &Path) -> bool {
    // Must be a file
    if !path.is_file() {
        return false;
    }

    // Get file and parent directory name
    let name = match path
        .file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.to_string())
    {
        Some(n) => n.to_lowercase(),
        None => return false,
    };
    let parent = match path.parent() {
        Some(p) => p,
        None => return false,
    };
    let parent_name = match parent
        .file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.to_string())
    {
        Some(n) => n.to_lowercase(),
        None => return false,
    };

    // Executable check on Unix
    #[cfg(unix)]
    if let Ok(meta) = path.metadata() {
        use std::os::unix::fs::MetadataExt;
        if (meta.mode() & 0o111) > 0 {
            return true;
        }
    }

    // Whitelist of parent directory names and binary suffixes
    let parents = ["bin", "binary", "run"];
    let suffixes = [".exe", ".x86", ".x86_64", ".bin", ".linux", "64"];

    parent_name == name || parents.iter().any(|n| &parent_name == n) || {
        suffixes.iter().any(|e| name.ends_with(e))
    }
}
