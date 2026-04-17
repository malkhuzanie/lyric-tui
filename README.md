# lyric-tui (lyt)

`lyric-tui` is a robust and responsive terminal user interface (TUI) for fetching and displaying lyrics from your favorite music players.

Written in Rust, it solves the fragility of traditional regex-based web scrapers by utilizing the highly reliable LRCLIB API (with fallback capability to Genius). It connects directly to your desktop's native media controllers via MPRIS (Linux), GSMTC (Windows), and MediaRemote (macOS).

## Features

- **Karaoke-Style Auto-Scrolling:** Parses time-synced `.lrc` lyrics to automatically scroll and highlight the currently playing line in real-time.
    
- **Zero-Block Async:** Built on `tokio`, the UI remains completely responsive at 60fps even while waiting for network requests.
    
- **Robust Provider Fallbacks:** Relies on the LRCLIB REST API by default for synchronized lyrics, with the option to dynamically switch to a Genius text fallback.
    
- **Manual Search & Alignment:** Easily search for arbitrary songs dynamically or toggle center/left text alignment to fit your setup.
    
- **Local Caching:** Automatically caches fetched lyrics locally using native XDG directories to save bandwidth and load instantly on replays.
    
- **Modern TUI:** Built with `ratatui`, featuring a clean, rounded-border aesthetic, progress bars, responsive text wrapping, and interactive popup menus.
    
- **Cross-Platform & Live Player Switching:** Dynamically detect and switch between active media clients at runtime using a built-in menu.
    

## Quick Start

Ensure you have Rust and Cargo installed. (Linux users will also need D-Bus development headers, e.g., `libdbus-1-dev`).

```
# Clone the repository
git clone [https://gitlab.com/malkhuzanie/lyric-tui.git](https://gitlab.com/malkhuzanie/lyric-tui.git)
cd lyric-tui

# Build the release binary
cargo build --release

# Run the app
./target/release/lyt
```

By default, `lyt` will automatically attach to any active media player (Spotify, VLC, Chrome, etc.).

For more detailed build instructions, including cross-compiling for Windows using `cargo-xwin` or building on macOS, please see our [Installation Guide](docs/installation.md "null").

## Usage & CLI Arguments

You can force `lyt` to listen to a specific media player identity on startup by passing it as an argument:

```
lyt "Spotify"
```

The application comes with built-in manual pages and help documentation.

- View CLI options: `lyt --help`
    
- View the man page (if installed): `man lyt`
    

## Keyboard Controls

A comprehensive help menu is built directly into the application. You can view it by pressing `?` or `h` while the application is running.

|Key|Action|
|---|---|
|`q` / `Esc`|Quit the application (or close the current popup)|
|`j` / `Down`|Scroll lyrics down|
|`k` / `Up`|Scroll lyrics up|
|`a`|Toggle auto-scroll on/off|
|`p`|Toggle lyric provider (LRCLIB vs Genius)|
|`c`|Toggle text alignment (Left / Center)|
|`s`|Open manual search prompt. Type "Artist - Title" and press Enter.|
|`l`|Open the runtime player selection menu to switch the media target.|
|`r`|Force a network reload (bypasses cache)|
|`?` / `h`|Show help menu|

## Documentation

We have comprehensive documentation available in the `docs/` folder:

- [**Installation Guide**](docs/installation.md "null") - Deep dive into compiling natively and cross-compiling for Linux, Windows, and macOS.
    
- [**Configuration Guide**](docs/configuration.md "null") - Learn how to customize colors, default providers, and network timeouts using `~/.config/lyric-tui/config.toml`.
    
- [**Architecture Guide**](docs/architecture.md "null") - A high-level overview of how the async event loops and media monitors work.
    
- [**Developer Guide**](docs/development.md "null") - Instructions for contributing, logging, and adding new lyric providers.
    

## Logs and Caching

- **Caching:** Lyrics are automatically saved in your user cache directory (e.g., `~/.cache/lyric-tui/`). You can clear this directory safely at any time to remove saved lyrics.
    
- **Logs:** A `lyric-tui.log` file is generated in the directory where the app is executed, providing detailed debug information mapping out the metadata and requests.
