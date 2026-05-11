use crossterm::event::KeyEvent;
use serde::{Deserialize, Serialize};
use std::time::Duration;

// ── Config (persisted to disk) ────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
pub enum Provider {
    #[default]
    Lrclib,
    Genius,
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
pub enum LyricAlignment {
    #[default]
    Left,
    Center,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct CoreConfig {
    pub default_player: Option<String>,
    pub network_timeout_secs: u64,
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            default_player: None,
            network_timeout_secs: 10,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct ViewConfig {
    pub default_provider: Provider,
    pub alignment: LyricAlignment,
    pub playhead_symbol: String,
    pub max_width: u16,
    pub max_lines: u16,
    pub full_screen: bool,
}

impl Default for ViewConfig {
    fn default() -> Self {
        Self {
            default_provider: Provider::Lrclib,
            alignment: LyricAlignment::Left,
            playhead_symbol: "▶ ".to_string(),
            max_width: 80,
            max_lines: 24,
            full_screen: false,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ThemeConfig {
    /// Primary background color (default: #1c1c20 / charcoal)
    pub background: Option<String>,
    /// Primary text color (default: #e8e1d2 / ivory)
    pub text: Option<String>,
    /// Secondary/dimmed text (default: #a09889 / linen)
    pub text_dim: Option<String>,
    /// Tertiary/very dim text - timestamps, separators (default: #5a5854 / slate)
    pub text_muted: Option<String>,
    /// Accent color - active line highlight (default: #d4af5f / gold)
    pub accent: Option<String>,
    /// Progress bar fill (default: #b48237 / amber)
    pub progress_fill: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct Config {
    pub core: CoreConfig,
    pub view: ViewConfig,
    pub theme: ThemeConfig,
}

// ── Domain types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct TrackInfo {
    pub artist: String,
    pub title: String,
    pub album: String,
    pub length: Option<Duration>,
}

#[derive(Debug, Clone)]
pub struct LyricLine {
    pub start_time: Option<Duration>,
    pub text: String,
}

// ── Event bus (inbound — from OS threads / tasks → main loop) ─────────────────

pub enum AppEvent {
    Input(KeyEvent),
    TrackChanged(TrackInfo),
    PositionUpdated(Duration),
    /// Lyrics have been fetched. Includes parsed lines and optionally the raw text
    /// for dual-caching purposes (when manual search needs to cache under alternate metadata).
    LyricsFetched {
        lines: Vec<LyricLine>,
        raw_text: Option<String>,
    },
    Tick,
    PlayersDiscovered(Vec<String>),
}

// ── Command bus (outbound — from event handler → main loop orchestration) ─────
//
// `handle_key_event` no longer spawns tasks directly. Instead it returns an
// optional `AppCommand` that `main.rs` inspects and acts on. This keeps all
// async orchestration (tokio::spawn, channel sends) in one place.

#[derive(Debug)]
pub enum AppCommand {
    /// Fetch lyrics for the given artist/title, optionally bypassing the cache.
    /// `alternate_artist` and `alternate_title` are optional metadata to cache the result
    /// under (in addition to the primary metadata), used for dual-caching when manual
    /// search finds lyrics that should also be cached under the player's original metadata.
    FetchLyrics {
        artist: String,
        title: String,
        force_refresh: bool,
        alternate_artist: Option<String>,
        alternate_title: Option<String>,
    },
    /// Atomically switch the OS media monitor to a new player identity.
    SwitchPlayer(String),
    /// Clear the targeted player (return to auto-detect).
    ClearPlayerTarget,
    /// Persist the current `Config` to disk and quit.
    Quit,
}

// ── App mode ──────────────────────────────────────────────────────────────────

#[derive(Default, Debug, PartialEq)]
pub enum AppMode {
    #[default]
    Normal,
    Search,
    Help,
    SelectPlayer,
}

// ── Sub-structs ───────────────────────────────────────────────────────────────

/// Everything that relates to *what is currently playing*.
#[derive(Default)]
pub struct PlaybackState {
    pub track: Option<TrackInfo>,
    pub position: Duration,
    pub active_line: usize,
}

/// Everything that relates to *how the lyrics pane is displayed*.
pub struct ViewState {
    pub scroll: u16,
    pub max_scroll: u16,
    pub viewport_height: u16,
    pub auto_scroll: bool,
    pub manual_active_line: usize,
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
            scroll: 0,
            max_scroll: 0,
            viewport_height: 0,
            auto_scroll: true, // on by default
            manual_active_line: 0,
        }
    }
}

/// Everything that relates to the manual-search popup.
#[derive(Default)]
pub struct SearchState {
    pub mode: AppMode,
    pub input: String,
}

/// Everything that relates to the runtime player-selection popup.
#[derive(Default)]
pub struct PlayerState {
    pub players: Vec<String>,
    pub selected_idx: usize,
}

// ── Top-level App ─────────────────────────────────────────────────────────────

#[derive(Default)]
pub struct App {
    pub config: Config,
    pub playback: PlaybackState,
    pub view: ViewState,
    pub search: SearchState,
    pub players: PlayerState,
    pub lyrics: Vec<LyricLine>,
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    // ── Convenience accessors ─────────────────────────────────────────────────
    //
    // These thin helpers let the rest of the codebase keep using short paths
    // (e.g. `app.mode()`) without having to know which sub-struct owns the
    // field. They are the *only* public API surface for cross-cutting reads.

    #[inline]
    pub fn mode(&self) -> &AppMode {
        &self.search.mode
    }

    #[inline]
    pub fn set_mode(&mut self, mode: AppMode) {
        self.search.mode = mode;
    }

    // ── Scroll helpers ────────────────────────────────────────────────────────

    pub fn scroll_down(&mut self) {
        if self.view.auto_scroll {
            self.view.manual_active_line = self.playback.active_line;
            self.view.auto_scroll = false;
        }
        self.view.manual_active_line = self.view.manual_active_line.saturating_add(1).min(self.lyrics.len().saturating_sub(1));
        
        let half_viewport = self.view.viewport_height / 2;
        let target_scroll = (self.view.manual_active_line as u16).saturating_sub(half_viewport);
        self.view.scroll = target_scroll.min(self.view.max_scroll);
    }

    pub fn scroll_up(&mut self) {
        if self.view.auto_scroll {
            self.view.manual_active_line = self.playback.active_line;
            self.view.auto_scroll = false;
        }
        self.view.manual_active_line = self.view.manual_active_line.saturating_sub(1);
        
        let half_viewport = self.view.viewport_height / 2;
        let target_scroll = (self.view.manual_active_line as u16).saturating_sub(half_viewport);
        self.view.scroll = target_scroll.min(self.view.max_scroll);
    }

    pub fn toggle_auto_scroll(&mut self) {
        self.view.auto_scroll = !self.view.auto_scroll;
        if self.view.auto_scroll {
            let half_viewport = self.view.viewport_height / 2;
            let target_scroll = (self.playback.active_line as u16).saturating_sub(half_viewport);
            self.view.scroll = target_scroll.min(self.view.max_scroll);
        } else {
            self.view.manual_active_line = self.playback.active_line;
        }
    }

    /// Must be called after `self.lyrics` is replaced so that `max_scroll` and
    /// the current `scroll` offset remain consistent.
    pub fn recalculate_scroll(&mut self) {
        let lines = self.lyrics.len() as u16;
        // max_scroll: last line sits at the bottom edge of the viewport — not
        // the mid-point. (The /2 bug noted in the architectural review.)
        self.view.max_scroll = lines.saturating_sub(self.view.viewport_height);
        self.view.scroll = self.view.scroll.min(self.view.max_scroll);
    }

    // ── Playback position update ──────────────────────────────────────────────

    pub fn update_position(&mut self, pos: Duration) {
        self.playback.position = pos;

        // Find the active line: last line whose start_time is ≤ current pos.
        let mut new_active = 0;
        for (i, line) in self.lyrics.iter().enumerate() {
            if let Some(time) = line.start_time {
                if time <= pos {
                    new_active = i;
                } else {
                    break;
                }
            }
        }
        self.playback.active_line = new_active;

        if self.view.auto_scroll {
            let half_viewport = self.view.viewport_height / 2;
            let target_scroll = (self.playback.active_line as u16).saturating_sub(half_viewport);
            self.view.scroll = target_scroll.min(self.view.max_scroll);
        }
    }

    // ── Lyrics reset helper ───────────────────────────────────────────────────
    //
    // Called whenever a new fetch is kicked off so the UI immediately shows a
    // status message instead of stale lyrics.

    pub fn set_status_lyric(&mut self, msg: impl Into<String>) {
        self.lyrics = vec![LyricLine {
            start_time: None,
            text: msg.into(),
        }];
        self.view.scroll = 0;
    }
}
