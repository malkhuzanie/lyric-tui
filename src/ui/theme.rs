/// lyric-tui Theme
///
/// A single source of truth for every color and style used in the UI.
/// Inspired by the warm tones of vinyl liner notes and audiophile print media:
/// deep charcoal backgrounds, ivory text, and restrained gold accents.
///
/// Nothing in the other ui modules should contain a raw `Color::*` or
/// `Style::default()` call — import from here instead.
///
/// Theme colors are loaded from config at startup. If not provided, defaults are used.
use ratatui::style::{Color, Modifier, Style};
use std::sync::OnceLock;

// ── Theme State (initialized at startup from config) ────────────────────────

static THEME: OnceLock<ThemeColors> = OnceLock::new();

/// Runtime theme colors initialized from config
#[derive(Debug, Clone)]
struct ThemeColors {
    /// Primary background color
    background: Color,
    /// Primary text color
    text: Color,
    /// Secondary/dimmed text
    text_dim: Color,
    /// Tertiary/very dim text (timestamps, separators)
    text_muted: Color,
    /// Accent color - active line highlight
    accent: Color,
    /// Progress bar fill
    progress_fill: Color,
    /// Dark fill for the progress bar trough (fixed)
    progress_trough: Color,
    /// Subtle border colour for inactive panels
    border_dim: Color,
    /// Brighter border for the lyrics panel (primary focus area)
    border_focus: Color,
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self {
            /// Deep charcoal — primary background / border colour
            background: Color::Rgb(28, 28, 32),      // #1c1c20

            /// Warm ivory — primary text
            text: Color::Rgb(232, 225, 210),         // #e8e1d2

            /// Muted linen — secondary / dimmed text
            text_dim: Color::Rgb(160, 152, 138),     // #a09889

            /// Deep slate — tertiary / very-dim text (timestamps, separators)
            text_muted: Color::Rgb(90, 88, 84),      // #5a5854

            /// Warm gold — accent / active line highlight
            accent: Color::Rgb(212, 175, 95),        // #d4af5f

            /// Amber — progress bar fill
            progress_fill: Color::Rgb(180, 130, 55), // #b48237

            /// Dark fill for the progress bar trough
            progress_trough: Color::Rgb(50, 48, 44), // #322c2c

            /// Subtle border colour for inactive panels
            border_dim: Color::Rgb(60, 58, 54),      // #3c3a36

            /// Brighter border for the lyrics panel (primary focus area)
            border_focus: Color::Rgb(100, 95, 80),   // #645f50
        }
    }
}

// ── Hex color parsing helper ───────────────────────────────────────────────

/// Parse a hex color string (e.g., "#e8e1d2" or "e8e1d2") to an RGB Color
fn parse_hex_color(hex: &str) -> Result<Color, String> {
    let hex = hex.trim_start_matches('#');
    
    if hex.len() != 6 {
        return Err(format!("Invalid hex color '{}': must be 6 characters", hex));
    }
    
    let r = u8::from_str_radix(&hex[0..2], 16)
        .map_err(|_| format!("Invalid hex color '{}': invalid red component", hex))?;
    let g = u8::from_str_radix(&hex[2..4], 16)
        .map_err(|_| format!("Invalid hex color '{}': invalid green component", hex))?;
    let b = u8::from_str_radix(&hex[4..6], 16)
        .map_err(|_| format!("Invalid hex color '{}': invalid blue component", hex))?;
    
    Ok(Color::Rgb(r, g, b))
}

/// Initialize theme from config. Called once at startup.
pub fn init_from_config(theme_config: &crate::app::ThemeConfig) {
    let mut colors = ThemeColors::default();
    
    // Parse and apply each optional config value
    if let Some(bg) = &theme_config.background {
        match parse_hex_color(bg) {
            Ok(color) => colors.background = color,
            Err(e) => log::warn!("Invalid background color in config: {}", e),
        }
    }
    if let Some(text) = &theme_config.text {
        match parse_hex_color(text) {
            Ok(color) => colors.text = color,
            Err(e) => log::warn!("Invalid text color in config: {}", e),
        }
    }
    if let Some(text_dim) = &theme_config.text_dim {
        match parse_hex_color(text_dim) {
            Ok(color) => colors.text_dim = color,
            Err(e) => log::warn!("Invalid text_dim color in config: {}", e),
        }
    }
    if let Some(text_muted) = &theme_config.text_muted {
        match parse_hex_color(text_muted) {
            Ok(color) => colors.text_muted = color,
            Err(e) => log::warn!("Invalid text_muted color in config: {}", e),
        }
    }
    if let Some(accent) = &theme_config.accent {
        match parse_hex_color(accent) {
            Ok(color) => colors.accent = color,
            Err(e) => log::warn!("Invalid accent color in config: {}", e),
        }
    }
    if let Some(progress_fill) = &theme_config.progress_fill {
        match parse_hex_color(progress_fill) {
            Ok(color) => colors.progress_fill = color,
            Err(e) => log::warn!("Invalid progress_fill color in config: {}", e),
        }
    }
    
    let _ = THEME.set(colors);
}

fn get_theme() -> &'static ThemeColors {
    THEME.get_or_init(ThemeColors::default)
}

// ── Composite Styles ───────────────────────────────────────────────────────

/// Border style for every panel (shared colour, overridden per panel below)
pub fn border_dim() -> Style {
    Style::default().fg(get_theme().border_dim)
}

pub fn border_focus() -> Style {
    Style::default().fg(get_theme().border_focus)
}

/// Track title in the header
pub fn track_title() -> Style {
    Style::default()
        .fg(get_theme().text)
        .add_modifier(Modifier::BOLD)
}

/// Artist name in the header
pub fn track_artist() -> Style {
    Style::default()
        .fg(get_theme().accent)
        .add_modifier(Modifier::BOLD)
}

/// Album name in the header — subtler
pub fn track_album() -> Style {
    Style::default().fg(get_theme().text_dim)
}

/// The currently-singing lyric line
pub fn lyric_active() -> Style {
    Style::default()
        .fg(get_theme().accent)
        .add_modifier(Modifier::BOLD)
}

/// A synced lyric line that has already passed
pub fn lyric_past() -> Style {
    Style::default().fg(get_theme().text_muted)
}

/// An upcoming synced lyric line
pub fn lyric_future() -> Style {
    Style::default().fg(get_theme().text_dim)
}

/// Plain (unsynced) lyric text
pub fn lyric_plain() -> Style {
    Style::default().fg(get_theme().text_dim)
}

/// Timestamp prefix — always very dim
pub fn timestamp() -> Style {
    Style::default().fg(get_theme().text_muted)
}

/// The playhead indicator glyph on the active line
pub fn playhead() -> Style {
    Style::default().fg(get_theme().accent).add_modifier(Modifier::BOLD)
}

/// Status bar key-hint labels (the key itself)
pub fn hint_key() -> Style {
    Style::default()
        .fg(get_theme().accent)
        .add_modifier(Modifier::BOLD)
}

/// Status bar key-hint descriptions
pub fn hint_desc() -> Style {
    Style::default().fg(get_theme().text_muted)
}

/// Auto-scroll indicator when ON
pub fn autoscroll_on() -> Style {
    Style::default().fg(get_theme().accent).add_modifier(Modifier::BOLD)
}

/// Auto-scroll indicator when OFF
pub fn autoscroll_off() -> Style {
    Style::default().fg(get_theme().text_muted)
}

/// Progress bar trough (unfilled background)
pub fn gauge_trough() -> Color {
    get_theme().progress_trough
}

/// Progress bar fill colour
pub fn gauge_fill() -> Style {
    Style::default().fg(get_theme().progress_fill).bg(get_theme().progress_trough)
}

/// Progress time label
pub fn gauge_label() -> Style {
    Style::default().fg(get_theme().text).add_modifier(Modifier::BOLD)
}

