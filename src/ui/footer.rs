/// Footer widget — playback progress bar.
///
/// Layout:
///
///   ╭──────────────────────────────────────────────────────────────╮
///   │  ▊▊▊▊▊▊▊▊▊▊▊▊▊▊▊▊▊▊▊▊▊▊▊▊▊▊▊▊▊▊▊▊  03:21 / 04:47             │
///   ╰──────────────────────────────────────────────────────────────╯
///
/// The bar uses the amber/trough palette.  The time label is right-aligned
/// inside the gauge label so it always reads clearly.
use crate::app::App;
use ratatui::{
    layout::Rect,
    style::Style,
    text::{Span, Line},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use super::theme;

/// Renders the progress bar footer into `area`.
pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let (ratio, label) = progress_state(app);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(theme::border_dim());

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    if inner_area.width == 0 {
        return;
    }

    let label_width = label.chars().count() + 4;
    let max_bar_width = (inner_area.width as usize).saturating_sub(label_width);

    let filled_len = ((max_bar_width as f64) * ratio).round() as usize;
    let empty_len = max_bar_width.saturating_sub(filled_len);

    let filled_str = "▊".repeat(filled_len);
    let empty_str = "▊".repeat(empty_len);

    let filled_span = Span::styled(filled_str, theme::gauge_fill());
    let empty_span = Span::styled(
        empty_str,
        Style::default().fg(theme::border_dim().fg.unwrap_or(ratatui::style::Color::DarkGray)),
    );

    let total_used = filled_len + empty_len + label.chars().count();
    let padding_count = (inner_area.width as usize).saturating_sub(total_used);
    let padding_str = " ".repeat(padding_count);

    let label_span = Span::styled(format!("{}{}", padding_str, label), theme::gauge_label());

    let line = Line::from(vec![filled_span, empty_span, label_span]);
    
    let paragraph = Paragraph::new(line)
        .style(Style::default().bg(theme::gauge_trough()));

    f.render_widget(paragraph, inner_area);
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
        } else {
            // Duration unknown (e.g., manual search) — show elapsed time only.
            let pos = app.playback.position.as_secs();
            let label = format!("{:02}:{:02}  /  —", pos / 60, pos % 60);
            return (0.0, label);
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

