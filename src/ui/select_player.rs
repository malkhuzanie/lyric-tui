/// Player selection popup widget.
///
/// Displayed in `AppMode::SelectPlayer`.  Lists all discovered media players
/// and highlights the currently-selected entry in gold.
use ratatui::{
    layout::{Alignment, Rect, Constraint, Flex, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Padding, Paragraph},
    Frame,
};

use crate::app::App;
use super::theme;

pub fn render(f: &mut Frame, app: &App, screen_area: Rect) {
    let block = Block::default()
        .title(" Select Media Player ")
        .borders(Borders::ALL)
        .border_style(theme::border_focus())
        .padding(Padding::symmetric(2, 1));

    let mut max_len = 22; // "No players discovered…" width
    for player in &app.players.players {
        max_len = max_len.max(player.len() + 4); // " > " + name + " "
    }

    let actual_popup_width = (max_len as u16 + 6) // borders + padding
        .min(screen_area.width.saturating_sub(4));

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

    let actual_popup_height = (lines.len() as u16 + 4) // content + borders + padding
        .min(screen_area.height.saturating_sub(2));

    let vertical_layout = Layout::vertical([Constraint::Length(actual_popup_height)])
        .flex(Flex::Center)
        .split(screen_area);

    let popup_area = Layout::horizontal([Constraint::Length(actual_popup_width)])
        .flex(Flex::Center)
        .split(vertical_layout[0])[0];

    f.render_widget(Clear, popup_area);

    let p = Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Left);

    f.render_widget(p, popup_area);
}
