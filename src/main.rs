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

    // Sync filesystem, run game
    fs::sync_fs();
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
    /// Tell Steam to install game.
    fn install(&self) {
        println!("Prompting to install game through Steam...");
        steam::invoke_steam_install(self.id);
    }

    /// Tell Steam to run game.
    fn run(&self) {
        println!("Starting game through Steam...");
        steam::invoke_steam_run(self.id);
    }
}

impl Default for Game {
    fn default() -> Self {
        // Find Steam games dir
        let steam_games = steam::find_steam_games_dir();

        // TODO: dynamically find Glitchball game in list of steam dirs, instead of just taking
        // first directory

        // Find placeholder game directory and binary
        let mut dir = steam_games[0].clone();
        dir.push("Glitchball/");
        let mut bin = dir.clone();
        #[cfg(not(macos))]
        bin.push("glitchball_linux.x86_64");
        #[cfg(macos)]
        bin.push("Glitchball.app");

        Self {
            id: 823470,
            path: GamePath { dir, bin },
        }
    }
}

/// Select game.
fn select_game() -> GamePath {
    // Find game directories
    let files = steam::find_steam_game_dirs();
    let game_items = skim_game_file_items(&files);

    let selected = select(game_items, "Select game").expect("did not select game");
    let dir: PathBuf = selected.into();
    let game_name = dir.file_name().unwrap().to_str().unwrap_or("?");

    let bins = steam::find_game_bins(&dir);
    if bins.is_empty() {
        panic!("No game files found");
    }
    let game_items = skim_game_file_items(&bins);

    let prompt = format!("Select binary ({})", game_name);
    let selected = match select(game_items, &prompt) {
        Some(g) => g,
        None => return select_game(),
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
