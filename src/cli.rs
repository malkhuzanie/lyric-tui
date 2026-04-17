// src/cli.rs
use clap::Parser;

/// A robust terminal user interface (TUI) for fetching and displaying lyrics.
#[derive(Parser, Debug)]
#[command(name = "lyt", version = "0.1.0", about, long_about = None)]
pub struct Cli {
    /// Force lyt to listen to a specific media player (e.g., "Spotify", "mpv")
    #[arg(index = 1)]
    pub target_player: Option<String>,
}

