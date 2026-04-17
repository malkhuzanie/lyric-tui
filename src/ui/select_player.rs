/// Player selection popup widget.
///
/// Displayed in `AppMode::SelectPlayer`.  Lists all discovered media players
/// and highlights the currently-selected entry in gold.
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::App;
use super::theme;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Select Media Player ")
        .borders(Borders::ALL)
        .border_style(theme::border_focus());

    let lines: Vec<Line<'_>> = if app.players.players.is_empty() {
        vec![Line::from(Span::styled(
            "No players discovered…",
            theme::lyric_plain(),
        ))]
    } else {
        app.players
            .players
            .iter()
            .enumerate()
            .map(|(i, name)| {
                if i == app.players.selected_idx {
                    Line::from(Span::styled(
                        format!(" > {} ", name),
                        Style::default().fg(Color::Yellow),
                    ))
                } else {
                    Line::from(Span::styled(
                        format!("   {} ", name),
                        theme::lyric_plain(),
                    ))
                }
            })
            .collect()
    };

    let p = Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Left);

    f.render_widget(Clear, area);
    f.render_widget(p, area);
}
