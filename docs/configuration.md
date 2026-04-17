# Configuration Guide

`lyric-tui` utilizes a `config.toml` file to manage user preferences, network settings, and custom theming. The application is designed to work perfectly out-of-the-box, so all of these settings are completely optional.

## File Locations

The configuration file is automatically generated the first time you run the app. It is placed in the standard configuration directory for your host operating system:

- **Linux/BSD:** `~/.config/lyric-tui/config.toml` (Follows XDG Base Directory specification)
    
- **Windows:** `%APPDATA%\tomato\lyric-tui\config\config.toml`
    
- **macOS:** `~/Library/Application Support/com.tomato.lyric-tui/config.toml`
    

## Configuration Structure

The configuration is divided into three logical blocks: `[core]`, `[view]`, and `[theme]`. If a block or field is missing from the file, the application will silently fall back to its internal defaults.

### Example `config.toml`

```
[core]
# Optional: Force the app to attach to a specific player on startup.
# Leave commented out or empty to auto-detect the active player.
default_player = "Spotify"

# How long to wait for lyric APIs before giving up (in seconds).
# Helpful for slower internet connections.
network_timeout_secs = 15

[view]
# Which lyric provider to use by default ("Lrclib" or "Genius")
default_provider = "Lrclib"

# The default text alignment for the lyrics ("Left" or "Center")
alignment = "Left"

# Custom indicator for the active line when left-aligned
playhead_symbol = "▶"

[theme]
# Colors must be standard 6-character hex codes (e.g., "#RRGGBB").
# If a color is omitted, the default warm audiophile palette is used.
# Each field below controls a specific UI element:

background = "#1C1C20"      # Primary background color of all panels
text = "#E8E1D2"            # Main text (current/future lyrics, track titles)
text_dim = "#A0988A"        # Secondary text (upcoming lyrics, album names)
text_muted = "#5A5854"      # Tertiary text (past lyrics, timestamps)
accent = "#D4AF5F"          # Accent highlights (active line, playhead, key hints)
progress_fill = "#B48237"   # Progress bar fill color
```

## Field Definitions & Fallbacks

### `[core]` Settings

These manage the background engines and network requests.

- **`default_player`** _(String, optional)_: The exact name of the media player (e.g., "Spotify", "VLC", "mpv") you want `lyric-tui` to track automatically on boot. Fallback: `None` (auto-detects the first active player).
    
- **`network_timeout_secs`** _(Integer)_: How long the HTTP client will wait for a response from the lyrics APIs before aborting. Increase this if you frequently see "Failed to fetch lyrics" errors on slow connections. Fallback: `10`.
    

### `[view]` Settings

These manage how the lyrics are displayed. **Note:** Some of these fields are automatically updated by the app when you use keyboard shortcuts (like `c` for alignment or `p` for provider).

- **`default_provider`** _(String)_: The service to hit first. Expected `"Lrclib"` or `"Genius"`. Fallback: `"Lrclib"`.
    
- **`alignment`** _(String)_: Expected `"Left"` or `"Center"`. Fallback: `"Left"`.
    
- **`playhead_symbol`** _(String)_: A custom string (usually 1-2 characters or an emoji) used to mark the active line when left-aligned. Fallback: `"▶ "`.
    

### `[theme]` Settings

Personalize the terminal UI colors. All theme properties are optional hex color strings in the format `"#RRGGBB"` (e.g., `"#FF00AA"`). If a color is omitted, the default warm audiophile palette is used. The `[theme]` block itself is entirely optional—if missing, all default colors apply.

**Color Semantics:**

- **`background`** _(String, optional)_: Primary background color of all panels. Fallback: `#1C1C20` (charcoal).
    
- **`text`** _(String, optional)_: Main text color for current and future lyrics, track titles, and artist names. Fallback: `#E8E1D2` (ivory).
    
- **`text_dim`** _(String, optional)_: Secondary text color for upcoming lyrics, album names, and inactive UI elements. Fallback: `#A0988A` (linen).
    
- **`text_muted`** _(String, optional)_: Tertiary text color for already-played lyrics, timestamps, separators, and very dim UI hints. Fallback: `#5A5854` (slate).
    
- **`accent`** _(String, optional)_: Accent color for the active/currently-playing lyric line, the playhead indicator, key hints in the status bar, and other interactive highlights. Fallback: `#D4AF5F` (gold).
    
- **`progress_fill`** _(String, optional)_: Color for the playback progress bar fill (the "played" portion). Fallback: `#B48237` (amber).

**Example with Nord theme colors:**

```toml
[theme]
background = "#2E3440"      # Nord theme background
text = "#ECEFF4"            # Nord theme foreground
text_dim = "#D8DEE9"        # Nord theme lighter gray
text_muted = "#4C566A"      # Nord theme darker gray
accent = "#88C0D0"          # Nord theme cyan
progress_fill = "#81A1C1"   # Nord theme blue
```
