/// Header widget — displays the currently-playing track.
///
/// Layout (single row inside a bordered panel):
///
///   ╭──────────────────────────────────────────────────────────────╮
///   │  ◆ ARTIST NAME  —  Track Title  ·  Album Name                │
///   ╰──────────────────────────────────────────────────────────────╯
///
/// The diamond (◆) is a fixed gold accent that anchors the left edge.
/// A long em-dash separates artist from title for a typographically
/// clean look reminiscent of vinyl label typography.
use crate::app::App;
use ratatui::{
    layout::Alignment,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};
use ratatui::layout::Rect;

use super::theme;

/// Renders the track-info header into `area`.
pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let content = build_header_line(app);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(theme::border_dim());

    let paragraph = Paragraph::new(vec![content])
        .block(block)
        .alignment(Alignment::Left);

    f.render_widget(paragraph, area);
}

fn build_header_line(app: &App) -> Line<'static> {
    match &app.playback.track {
        Some(t) => Line::from(vec![
            Span::styled("  ◆ ", theme::track_artist()),
            Span::styled(t.artist.to_uppercase(), theme::track_artist()),
            Span::styled("  —  ", theme::track_album()),
            Span::styled(t.title.clone(), theme::track_title()),
            Span::styled("  ·  ", theme::track_album()),
            Span::styled(t.album.clone(), theme::track_album()),
            Span::styled("  ", theme::track_album()),
        ]),
        None => Line::from(vec![
            Span::styled("  ◆ ", theme::timestamp()),
            Span::styled("Waiting for a media player …", theme::timestamp()),
        ]),
    }
}

