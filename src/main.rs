mod steam;
mod util;

use std::path::PathBuf;
use std::process;

fn main() {
    util::report_unsupported_platform();

    // Define placeholder game and game to play
    let placeholder = Game::default();
    let game = GamePath::from_bin(
        "/home/timvisee/.steam/steam/steamapps/common/Stephen's Sausage Roll/Sausage.x86_64".into(),
    );

    // Placeholder game directory must exist
    if !placeholder.path.dir_exists() {
        eprintln!("Placeholder game 'Glitchball' not installed");
        eprintln!("Install game through Steam first, or repair game files");
        placeholder.install();
        process::exit(1);
    }

    // Prepare placeholder game
    println!("Preparing placeholder game...");
    placeholder.path.replace_contents_with_linked(&game);

    // Run game
    placeholder.run();
}

/// Represents a game with a path.
struct GamePath {
    /// Game root directory.
    dir: PathBuf,

    /// Game binary.
    bin: PathBuf,
}

impl GamePath {
    /// Construct instance from given game binary.
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
