use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::util;

/// Remove all contents in the given directory.
///
/// - Removes all directories and files.
/// - Symlinks are unlinked.
pub fn remove_dir_contents(dir: &Path) -> io::Result<()> {
    for entry in dir.read_dir()? {
        let entry = entry?;
        if fs::symlink_metadata(entry.path())?.is_dir() {
            fs::remove_dir_all(entry.path())?;
        } else {
            fs::remove_file(entry.path())?;
        }
    }
    Ok(())
}

/// List items in directory.
pub fn ls(dir: &Path) -> io::Result<Vec<PathBuf>> {
    Ok(dir
        .read_dir()?
        .filter_map(|entry| entry.map(|e| e.path()).ok())
        .collect())
}

/// Sync filesystem.
pub fn sync_fs() {
    #[cfg(target_os = "linux")]
    {
        eprintln!("Syncing filesystem...");
        if let Err(e) = util::invoke_cmd("sync") {
            eprintln!("Request to sync filesystem failed, ignoring: {:?}", e);
        }
    }
}
