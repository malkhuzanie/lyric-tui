# Developer Guide

Welcome to the `lyric-tui` codebase! This document outlines how to set up your development environment, navigate the project, and contribute effectively.

## Getting Started

1. **Clone the repository:**
    
    ```
    git clone https://gitlab.com/malkhuzanie/lyric-tui.git
    cd lyric-tui
    ```
    
2. **Run in debug mode:**
    
    ```
    cargo run
    ```
    
    _Note: Because `lyt` takes over the terminal window, `println!` macros will corrupt the UI. Use the logging system instead._
    

## Logging & Debugging

`lyric-tui` utilizes the `log` and `simplelog` crates.

Whenever you run the application, a `lyric-tui.log` file is automatically generated in the working directory. This log captures:

- Network requests and API errors.
    
- Cache hits/misses.
    
- Raw metadata payloads from the media monitors (helpful for debugging weird track names from certain players).
    
- Application state transitions.
    

To log your own debug messages, use standard log macros:

```
log::debug!("Some variable: {}", my_var);
log::warn!("Something might be broken!");
```

You can safely `tail -f lyric-tui.log` in a separate terminal tab while the application is running.

## Project Structure

- `src/app.rs`: Defines the central `App` state, data models (`LyricLine`, `TrackInfo`), and event/command enums.
    
- `src/main.rs`: The async entry point. Sets up the UI, handles the `tokio` runtime, and orchestrates the event loop.
    
- `src/events.rs`: Pure, synchronous keyboard event handling. Mutates the `App` and returns side-effect `AppCommand`s.
    
- `src/media_monitor/`: OS-specific modules for detecting what is currently playing.
    
- `src/providers/`: Implementations for fetching lyrics from the web (`lrclib`, `genius`).
    
- `src/ui/`: Ratatui widget definitions and thematic styling.
    

## Adding a New Lyric Provider

If you want to add a new source for lyrics (e.g., Musixmatch, NetEase), you can easily integrate it by implementing the `LyricProvider` trait.

1. Create a new file in `src/providers/` (e.g., `src/providers/myprovider.rs`).
    
2. Implement the trait:
    
    ```
    use crate::app::LyricLine;
    use super::LyricProvider;
    use anyhow::Result;
    
    pub struct MyProvider;
    
    #[async_trait::async_trait]
    impl LyricProvider for MyProvider {
        async fn fetch(&self, artist: &str, title: &str, force_refresh: bool) -> Result<Vec<LyricLine>> {
            // 1. Check local cache (using directories crate)
            // 2. Fetch from your API using reqwest
            // 3. Parse into Vec<LyricLine>
            // 4. Save to local cache
            // 5. Return result
        }
    }
    ```
    
3. Add the provider to the `Provider` enum in `src/app.rs`.
    
4. Add the invocation logic in `fetch_lyrics` inside `src/main.rs`.
    

## UI Conventions & Theming

When building or modifying UI components in `src/ui/`:

1. **Never use raw colors:** Always import from `src/ui/theme.rs`. If you need a new color, define it in `theme.rs` first. This ensures the app maintains a consistent, warm aesthetic.
    
2. **Handle resizing:** Ensure your layout constraints use relative sizing (`Constraint::Percentage`, `Constraint::Min`) where possible so the app behaves well on small terminal windows.

