use std::env;
use std::path::{Path, PathBuf};

use crate::fs;

/// Invoke a Steam URL.
///
/// Give the path to invoke. `install/1` will invoke `steam://install/1`.
fn invoke_steam_url(path: &str) {
    open::that(&format!("steam://{}", path)).expect("failed to invoke Steam URL");
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
pub fn find_steam_games_dir() -> Vec<PathBuf> {
    #[allow(deprecated)]
    let home = env::home_dir().expect("unable to determine user home directory");

    // Build list of default steam game directories
    let mut dirs: Vec<PathBuf> = vec![];
    #[cfg(unix)]
    {
        let mut dir = home.clone();
        dir.push(".steam/steam/steamapps/common/");
        dirs.push(dir);
    }
    #[cfg(macos)]
    {
        let mut dir = home.clone();
        dir.push("Library/Application Support/Steam/steamapps/common/");
        dirs.push(dir);
    }
    #[cfg(windows)]
    {
        dirs.push(r#"C:\Program Files (x86)\Steam\steamapps\common\"#.into());
        dirs.push(r#"C:\Program Files\Steam\steamapps\common\"#.into());
    }

    // Get list of game directories, more dirs can be added here
    let mut dirs: Vec<PathBuf> = dirs.into_iter().filter(|d| d.is_dir()).collect();

    // Extend list with user defined directories configured in Steam
    let extra_dirs: Vec<PathBuf> = dirs
        .iter()
        .flat_map(|d| {
            // Scan directory and parent directory (./common/..)
            let mut dirs = vec![d.to_owned()];
            if let Some(parent) = d.parent() {
                dirs.push(parent.into());
            }
            dirs
        })
        .filter_map(|d| find_steam_games_dir_extras(d))
        .flatten()
        .collect();
    dirs.extend_from_slice(&extra_dirs);

    // Remove duplicates
    dirs.sort_unstable();
    dirs.dedup();

    dirs
}

/// Find additional Steam game directories, configured by the user.
fn find_steam_games_dir_extras(mut path: PathBuf) -> Option<Vec<PathBuf>> {
    // Append filename to path
    path.push("libraryfolders.vdf");

    // Load user library folder configuration, find library folders table
    let entry = steamy_vdf::load(path).ok()?;
    let table = entry.get("LibraryFolders")?.as_table()?;

    // List keys that are a number
    let keys: Vec<&String> = table
        .keys()
        .into_iter()
        .filter(|n| n.chars().filter(|c| !('0'..='9').contains(c)).count() == 0)
        .collect();

    // Grab library paths, directories must exist
    Some(
        keys.into_iter()
            .filter_map(|k| {
                table
                    .get(k)
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string().into())
            })
            .map(|mut d: PathBuf| {
                d.push("steamapps/common/");
                d
            })
            .filter(|d: &PathBuf| d.is_dir())
            .collect(),
    )
}

/// Find directories of steam games.
pub fn find_steam_game_dirs() -> Vec<PathBuf> {
    find_steam_games_dir()
        .into_iter()
        .flat_map(|d| {
            fs::ls(&d)
                .expect("failed to list Steam game dirs")
                .into_iter()
                .filter(|f| f.is_dir())
                .filter(|f| game_has_bins(f))
        })
        .collect()
}

/// Find game binaries.
pub fn find_game_bins(dir: &Path) -> Vec<PathBuf> {
    fs::ls(dir)
        .expect("failed to list Steam game dirs")
        .into_iter()
        .filter(|f| is_bin(&f))
        .collect()
}

/// Check whether a game has binaries.
pub fn game_has_bins(path: &Path) -> bool {
    !find_game_bins(path).is_empty()
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

    // Whitelist of parent directory names and binary suffixes
    let parents = ["bin", "binary", "run", "game"];
    let wl_suffix = [
        ".exe",
        ".x86",
        ".x86_64",
        ".bin",
        #[cfg(linux)]
        ".linux",
        "64",
        #[cfg(macos)]
        ".app",
    ];
    let bl_suffix = [".dll", ".lock", ".ds_store"];

    // Skip blacklisted
    if bl_suffix.iter().any(|e| name.ends_with(e)) {
        return false;
    }

    // Executables are binaries on Unix
    #[cfg(unix)]
    {
        if let Ok(meta) = path.metadata() {
            use std::os::unix::fs::MetadataExt;
            if (meta.mode() & 0o111) > 0 {
                return true;
            }
        }
    }

    // Check whitelist
    parent_name == name || parents.iter().any(|n| &parent_name == n) || {
        wl_suffix.iter().any(|e| name.ends_with(e))
    }
}
