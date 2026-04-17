# Architecture Guide

`lyric-tui` is built on a clean, concurrent architecture using `tokio` for async orchestration and `ratatui` for rendering. The application strictly separates state mutation from side effects, ensuring the UI runs at a buttery smooth 60fps regardless of network latency or slow D-Bus/COM responses.

## High-Level Diagram

```
┌─────────────────┐       ┌─────────────────┐       ┌─────────────────┐
│  Media Monitor  │       │ Keyboard Events │       │  Tick (60fps)   │
│ (OS Background) │       │   (crossterm)   │       │ (tokio interval)│
└────────┬────────┘       └────────┬────────┘       └────────┬────────┘
         │                         │                         │
         │ AppEvent                │ AppEvent                │ AppEvent
         ▼                         ▼                         ▼
      ┌───────────────────────────────────────────────────────────┐
      │                      MPSC Channel                         │
      └────────────────────────────┬──────────────────────────────┘
                                   │
                                   ▼
      ┌───────────────────────────────────────────────────────────┐
      │                        Main Loop                          │
      │ 1. Update State (`App`)                                   │
      │ 2. Process Commands (`AppCommand`) -> Spawn async tasks   │
      │ 3. Render UI (`ratatui`) if state changed                 │
      └───────────────────────────────────────────────────────────┘
```

## The Event Loop

The core of the application lives in `src/main.rs`. It relies on a multi-producer, single-consumer (`mpsc`) channel to funnel events from background threads into a single UI loop.

- **Event Bus (`AppEvent`):** Represents inbound data. Examples include `Input` (keypresses), `TrackChanged`, `PositionUpdated`, and `LyricsFetched`.
    
- **Command Bus (`AppCommand`):** Represents outbound side-effects. The keyboard event handler does _not_ spawn tasks directly. Instead, it mutates the state and returns an `AppCommand` (e.g., `FetchLyrics`, `SwitchPlayer`), which `main.rs` then executes. This keeps the event handler perfectly synchronous and highly testable.
    

## Media Monitors

To support multiple operating systems, media monitoring is abstracted into `src/media_monitor/`. A dedicated OS thread polls the native media controllers:

- **Linux (`mpris`):** Uses D-Bus to communicate with MPRIS-compatible players.
    
- **Windows (`windows` crate):** Uses the `GlobalSystemMediaTransportControlsSessionManager` (GSMTC) via COM interfaces.
    
- **macOS (`mediaremote-rs`):** Interfaces with Apple's private MediaRemote framework.
    

These threads run continuously, pushing `PositionUpdated` (for auto-scrolling) and `TrackChanged` events into the `mpsc` channel.

## State Management (`app.rs`)

The entire application state is consolidated into a single `App` struct. It is subdivided logically:

- `playback`: What is currently playing, position, and active lyric line.
    
- `view`: Scroll position, viewport height, and auto-scroll toggle.
    
- `search`: State for the manual search popup.
    
- `players`: State for the media player selection popup.
    
- `config`: User preferences (persisted to disk).
    

## Lyric Providers (`providers/`)

Lyrics fetching is standardized behind the `LyricProvider` async trait.

- **LrclibProvider:** The default provider. It connects to the `lrclib.net` REST API to fetch time-synced `.lrc` data.
    
- **GeniusProvider:** A fallback provider. It uses `scraper` to parse the Genius DOM for unsynced lyrics.
    

**Caching:** To save bandwidth and improve load times, all fetched lyrics are written directly to the OS-native cache directory (`~/.cache/lyric-tui/`) using the `directories` crate.

## UI Rendering (`ui/`)

The UI is purely a reflection of the `App` state. Layout logic is isolated to `src/ui/mod.rs`, which divides the terminal into three main sections:

1. `header.rs`: Track and artist info.
    
2. `lyrics.rs`: The main viewport (handles scrolling, active line highlighting, and timestamp formatting).
    
3. `footer.rs`: The progress bar gauge.

Other modules handle popups and overlays, such as `search.rs` (manual search popup), `select_player.rs` (player selection list), and `help.rs` (shortcut bindings).
    
All colors and visual modifiers are strictly managed in `src/ui/theme.rs`.

