#[macro_use]
extern crate clap;

mod steam;
mod util;

use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process;

use skim::{
    prelude::{SkimItemReader, SkimOptionsBuilder},
    Skim,
};

use clap::{App, Arg};

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

    // Select game
    let game = match matches.value_of("GAME") {
        Some(game) => GamePath::from_bin(game.into()),
        None => select_game(),
    };

    // Define placeholder game
    eprintln!("Using placeholder game: Glitchball");
    let placeholder = Game::default();

    // Placeholder game directory must exist
    if !placeholder.path.dir_exists() {
        eprintln!("Placeholder game 'Glitchball' not installed");
        eprintln!("Install game through Steam first, or repair game files");
        placeholder.install();
        process::exit(1);
    }

    // Prepare placeholder game
    eprintln!("Preparing game...");
    placeholder.path.replace_contents_with_linked(&game);

    // Sync filesystem
    util::sync_fs();

    // Run game
    placeholder.run();
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

    /// Check whether game directory exists.
    fn dir_exists(&self) -> bool {
        self.dir.is_dir()
    }

    /// Replace game with given other game.
    ///
    /// This deletes game files in the directory of this game, and creates proper links to the
    /// given game.
    fn replace_contents_with_linked(&self, replacement: &GamePath) {
        // Clear contents
        util::remove_dir_contents(&self.dir).expect("Failed to clear directory contents");

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
    /// Tell Steam to install game.
    fn install(&self) {
        println!("Prompting to install game through Steam...");
        steam::invoke_steam_install(self.id);
    }

    /// Tell Steam to run game.
    fn run(&self) {
        println!("Running game...");
        steam::invoke_steam_run(self.id);
    }
}

impl Default for Game {
    fn default() -> Self {
        // Find Steam games dir
        let steam_games = steam::find_steam_games_dir();

        // Find placeholder game directory and binary
        let mut dir = steam_games.clone();
        dir.push("Glitchball/");
        let mut bin = dir.clone();
        bin.push("glitchball_linux.x86_64");

        Self {
            id: 823470,
            path: GamePath { dir, bin },
        }
    }
}

/// Select game.
fn select_game() -> GamePath {
    // Get Steam directory
    let steam = steam::find_steam_games_dir();

    let files = util::ls(&steam).expect("failed to list Steam game dirs");

    // TODO: do not unwrap in here
    let files: Vec<String> = files
        .into_iter()
        .map(|d| d.to_str().unwrap().to_owned())
        .collect();

    let selected = select(&files, "Select game").expect("did not select game");

    let dir: PathBuf = selected.into();

    let bin = select_game_bin(&dir).expect("no game selected");

    GamePath { dir, bin }
}

/// Select game binary.
fn select_game_bin(dir: &Path) -> Option<PathBuf> {
    // TODO: do not unwrap
    let files = util::ls(&dir).unwrap();

    // TODO: do not unwrap in here
    let files: Vec<String> = files
        .into_iter()
        .map(|d| d.to_str().unwrap().to_owned())
        .collect();

    match select(&files, "Select game binary") {
        Some(file) => {
            let path: PathBuf = file.into();
            if path.is_file() {
                return Some(path);
            }
            if let Some(bin) = select_game_bin(&path) {
                return Some(bin);
            } else {
                return select_game_bin(dir);
            }
        }
        None => {
            return None;
        }
    }
}

/// Show an interactive selection view for the given list of `items`.
/// The selected item is returned.  If no item is selected, `None` is returned instead.
fn select(items: &[String], prompt: &str) -> Option<String> {
    let prompt = format!("{}: ", prompt);

    let options = SkimOptionsBuilder::default()
        .prompt(Some(&prompt))
        .height(Some("50%"))
        .multi(false)
        .build()
        .unwrap();

    let input = items.join("\n");

    // `SkimItemReader` is a helper to turn any `BufRead` into a stream of `SkimItem`
    // `SkimItem` was implemented for `AsRef<str>` by default
    let item_reader = SkimItemReader::default();
    let items = item_reader.of_bufread(Cursor::new(input));

    // `run_with` would read and show items from the stream
    let selected = Skim::run_with(&options, Some(items))
        .map(|out| out.selected_items)
        .unwrap_or_else(|| Vec::new());

    // Get the first selected, and return
    selected.iter().next().map(|i| i.output().to_string())
}
