# lyric-tui (lyt)

`lyric-tui` is a robust and responsive terminal user interface (TUI) for fetching and displaying lyrics from your favorite music players.

Written in Rust, it solves the fragility of traditional regex-based web scrapers by utilizing the highly reliable LRCLIB API (with fallback capability to Genius). It connects directly to your desktop's native media controllers via MPRIS (Linux), GSMTC (Windows), and MediaRemote (macOS).

## Features

- **Karaoke-Style Auto-Scrolling:** Parses time-synchronised \.lrc` files to scroll automatically and highlight the active lyric in real time.
- **Non-Blocking Asynchronous Architecture:** Built upon \tokio`, the interface maintains a fluid 60 frames per second, remaining entirely responsive whilst awaiting network requests.
- **Robust Provider Fallbacks:** Relies upon the LRCLIB REST API by default for synchronised lyrics, providing the facility to switch dynamically to a Genius text fallback should the primary provider fail.
- **Manual Search Facility:** Dynamically search for arbitrary tracks with ease.
- **Immersive Full-Screen Mode:** Toggle an expansive, borderless display that strips away all chrome and tracking logic to present only the lyrics.
- **Customisable Typography:** Toggle between left-aligned and centred text to suit your visual preference.
- **Intelligent Local Caching:** Automatically caches retrieved lyrics within native XDG directories; this conserves bandwidth and ensures instantaneous loading upon subsequent playbacks.
- **Contemporary Terminal Interface:** Built with `ratatui`, the application boasts a clean, rounded-border aesthetic, accompanied by progress bars, responsive text wrapping, and interactive pop-up menus.
- **Cross-Platform Compatibility & Live Player Switching:** Dynamically detect and transition between active media clients at runtime via an integrated menu, supporting MPRIS (Linux), GSMTC (Windows), and MediaRemote (macOS).

## Known Limitations & Disclaimers

- **Genius Provider:** Whilst implemented, the Genius service is heavily shielded by Cloudflare's automated security measures. Consequently, queries to Genius may frequently fail or be intercepted. LRCLIB remains the recommended default provider.
- **macOS Support:** The macOS daemon integration (\mediaremote-rs`) has been implemented; however, it is currently unverified and remains in a beta testing phase.

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
|`f`|Toggle full-screen mode|
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

## License
This project is distributed under the MIT License. For comprehensive legal details, please consult the LICENSE file residing within the repository root.