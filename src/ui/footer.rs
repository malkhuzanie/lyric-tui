/// Footer widget — playback progress bar.
///
/// Layout:
///
///   ╭──────────────────────────────────────────────────────────────╮
///   │  ████████████████████░░░░░░░░░░░░  03:21 / 04:47             │
///   ╰──────────────────────────────────────────────────────────────╯
///
/// The bar uses the amber/trough palette.  The time label is right-aligned
/// inside the gauge label so it always reads clearly.
use crate::app::App;
use ratatui::{
    layout::Rect,
    style::Style,
    text::Span,
    widgets::{Block, BorderType, Borders, Gauge},
    Frame,
};

use super::theme;

/// Renders the progress bar footer into `area`.
pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let (ratio, label) = progress_state(app);

    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Plain)
                .border_style(theme::border_dim()),
        )
        .gauge_style(theme::gauge_fill())
        // Override the default white bg that ratatui injects for the unfilled part
        .style(Style::default().bg(theme::gauge_trough()))
        .ratio(ratio)
        .label(Span::styled(label, theme::gauge_label()));

    f.render_widget(gauge, area);
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Returns `(ratio, label)` from the current app state.
fn progress_state(app: &App) -> (f64, String) {
    if let Some(track) = &app.playback.track {
        if let Some(length) = track.length {
            let pos = app.playback.position.as_secs();
            let len = length.as_secs();
            let ratio = if len > 0 {
                (pos as f64 / len as f64).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let label = format_time_pair(pos, len);
            return (ratio, label);
        }
    }
    (0.0, "  —  ".to_string())
}

fn format_time_pair(pos: u64, len: u64) -> String {
    format!(
        "{:02}:{:02}  /  {:02}:{:02}",
        pos / 60,
        pos % 60,
        len / 60,
        len % 60
    )
}

