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
        .filter(|f| f.is_file())
        .collect()
}
