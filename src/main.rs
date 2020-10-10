#[macro_use]
extern crate clap;

mod fs;
mod steam;
mod util;

use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::Arc;

use skim::{
    prelude::{SkimItemReceiver, SkimItemSender, SkimOptionsBuilder},
    AnsiString, Skim, SkimItem,
};

use clap::{App, Arg};

const PLACEHOLDER_GAME_NAME: &str = "Glitchball";
const PLACEHOLDER_GAME_ID: usize = 823470;
const PLACEHOLDER_GAME_DIR: &str = "Glitchball";

fn main() {
    // Handle CLI arguments
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name("GAME")
                .help("Path to binary of game to run")
                .required(false)
                .index(1),
        )
        .get_matches();

    util::report_unsupported_platform();

    // Find steam game dirs
    // TODO: use this throughout application, do not call multiple times
    let steam_dirs = steam::find_steam_dirs();

    // Find placeholder game
    let placeholder = find_placeholder_game(&steam_dirs);

    // Select game
    let game = match matches.value_of("GAME") {
        Some(game) => GamePath::from_bin(game.into()),
        None => select_game(&steam_dirs),
    };

    // Prepare placeholder game
    eprintln!("Preparing game...");
    placeholder.path.replace_contents_with_linked(&game);

    // Sync filesystem, run game
    fs::sync_fs();
    placeholder.run();
}

/// Find the placeholder game.
///
/// If there's an installation issue the user is prompted and the program quits.
fn find_placeholder_game(steam_dirs: &[PathBuf]) -> Game {
    eprintln!("Using placeholder game: {}", PLACEHOLDER_GAME_NAME);
    match steam::find_game_dir(&steam_dirs, PLACEHOLDER_GAME_DIR) {
        dirs if dirs.len() == 1 => Game::placeholder(dirs[0].clone()),
        dirs if dirs.is_empty() => {
            eprintln!("Placeholder game '{}' not installed", PLACEHOLDER_GAME_NAME);
            eprintln!(
                "Install game through Steam first, or repair game files, then run this again"
            );
            steam::invoke_steam_install(PLACEHOLDER_GAME_ID);
            steam::invoke_steam_validate(PLACEHOLDER_GAME_ID);
            process::exit(1);
        }
        dirs => {
            eprintln!(
                "Placeholder game '{}' has multiple install locations",
                PLACEHOLDER_GAME_NAME
            );

            // Remove installation directories
            eprintln!("Removing installation directories...");
            dirs.into_iter().for_each(|d| {
                // Remove directory contents
                if let Err(err) = fs::remove_dir_contents(&d) {
                    eprintln!(
                        "Failed to remove game installation directory contents, ignoring: {:?}",
                        err
                    );
                }

                // Remove directory itself
                if let Err(err) = std::fs::remove_dir(&d) {
                    eprintln!(
                        "Failed to remove game installation directory, ignoring: {:?}",
                        err
                    );
                }
            });

            // Uninstall through Steam, give user instruction
            eprintln!("Uninstall game through Steam, then run this again to reinstall");
            steam::invoke_steam_uninstall(PLACEHOLDER_GAME_ID);

            process::exit(1);
        }
    }
}

/// Path to a game.
struct GamePath {
    /// Game root directory.
    dir: PathBuf,

    /// Game binary.
    bin: PathBuf,
}

impl GamePath {
    /// From given binary path.
    ///
    /// This attempts to guess the game root directory.
    fn from_bin(path: PathBuf) -> Self {
        // Binary must exist, find root dir
        if !path.is_file() {
            panic!("given binary path is not a file");
        }
        let path = path
            .canonicalize()
            .expect("failed to canonicalize binary path");
        let dir = path
            .parent()
            .expect("failed to determine root directory of binary")
            .into();

        GamePath { dir, bin: path }
    }

    /// Replace game with given other game.
    ///
    /// This deletes game files in the directory of this game, and creates proper links to the
    /// given game.
    fn replace_contents_with_linked(&self, replacement: &GamePath) {
        // Clear contents
        fs::remove_dir_contents(&self.dir).expect("Failed to clear directory contents");

        // Link to replacement game
        std::os::unix::fs::symlink(&replacement.bin, &self.bin)
            .expect("failed to link given game to placeholder game");
    }
}

/// Represents a Steam game.
struct Game {
    /// Steam game ID.
    id: usize,

    /// Game path details.
    path: GamePath,
}

impl Game {
    /// Tell Steam to run game.
    fn run(&self) {
        println!("Starting game through Steam...");
        steam::invoke_steam_run(self.id);
    }

    /// Construct placeholder game at given path.
    fn placeholder(dir: PathBuf) -> Self {
        let mut bin = dir.clone();
        #[cfg(not(target_os = "macos"))]
        bin.push("glitchball_linux.x86_64");
        #[cfg(target_os = "macos")]
        bin.push("Glitchball.app");

        Self {
            id: PLACEHOLDER_GAME_ID,
            path: GamePath { dir, bin },
        }
    }
}

/// Select game.
fn select_game(steam_dirs: &[PathBuf]) -> GamePath {
    // Find game directories
    let game_dirs = steam::find_steam_game_dirs(steam_dirs);
    let game_items = skim_game_file_items(&game_dirs);

    let selected = select(game_items, "Select game").expect("did not select game");
    let dir: PathBuf = selected.into();
    let game_name = dir.file_name().unwrap().to_str().unwrap_or("?");

    let bins = steam::find_game_bins(&dir, 999999999);
    if bins.is_empty() {
        panic!("No game files found");
    }
    let game_items = skim_game_file_items(&bins);

    let prompt = format!("Select binary ({})", game_name);
    let selected = match select(game_items, &prompt) {
        Some(g) => g,
        None => return select_game(steam_dirs),
    };
    let bin: PathBuf = selected.into();

    GamePath { dir, bin }
}

/// Show an interactive selection view for the given list of `items`.
/// The selected item is returned.  If no item is selected, `None` is returned instead.
fn select(items: SkimItemReceiver, prompt: &str) -> Option<String> {
    let prompt = format!("{}: ", prompt);
    let options = SkimOptionsBuilder::default()
        .prompt(Some(&prompt))
        .height(Some("50%"))
        .multi(false)
        .build()
        .unwrap();

    let selected = Skim::run_with(&options, Some(items))
        .map(|out| out.selected_items)
        .unwrap_or_else(|| Vec::new());

    // Get the first selected, and return
    selected.iter().next().map(|i| i.output().to_string())
}

pub struct SkimGameFile {
    /// Game directory.
    dir: String,

    /// Game name.
    name: String,
}

impl SkimGameFile {
    /// Construct from given game directory.
    fn from(dir: &Path) -> Self {
        Self {
            dir: dir.to_str().unwrap().into(),
            name: dir.file_name().unwrap().to_str().unwrap().into(),
        }
    }
}

impl SkimItem for SkimGameFile {
    fn display(&self) -> Cow<AnsiString> {
        let s: AnsiString = self.name.clone().into();
        Cow::Owned(s)
    }

    fn text(&self) -> Cow<str> {
        (&self.name).into()
    }

    fn output(&self) -> Cow<str> {
        // Return full path
        (&self.dir).into()
    }
}

/// Generate skim `GameItem` from given paths.
fn skim_game_file_items(paths: &[PathBuf]) -> SkimItemReceiver {
    // Transform into skim game item and sort
    let mut paths: Vec<_> = paths.iter().map(|g| SkimGameFile::from(g)).collect();
    paths.sort_unstable_by(|a, b| a.name.cmp(&b.name));

    let (tx_item, rx_item): (SkimItemSender, SkimItemReceiver) =
        skim::prelude::bounded(paths.len());

    paths.into_iter().for_each(|g| {
        let _ = tx_item.send(Arc::new(g));
    });

    rx_item
}
