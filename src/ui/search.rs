use ratatui::{
    layout::{Alignment, Rect, Constraint, Flex, Layout},
    widgets::{Block, Borders, Clear, Padding, Paragraph},
    Frame,
};

use crate::app::App;
use super::theme;

pub fn render(f: &mut Frame, app: &App, screen_area: Rect) {
    let block = Block::default()
        .title(" Manual Search (Artist - Title) ")
        .borders(Borders::ALL)
        .border_style(theme::border_focus())
        .padding(Padding::symmetric(2, 1));

    let min_width = 40;
    let actual_popup_width = (app.search.input.len() as u16 + 6)
        .max(min_width)
        .min(screen_area.width.saturating_sub(4));

    let actual_popup_height = 5; // 1 line content + 2 padding + 2 borders

    let vertical_layout = Layout::vertical([Constraint::Length(actual_popup_height)])
        .flex(Flex::Center)
        .split(screen_area);

    let popup_area = Layout::horizontal([Constraint::Length(actual_popup_width)])
        .flex(Flex::Center)
        .split(vertical_layout[0])[0];

    let mut display_text = app.search.input.clone();
    display_text.push('█'); // simple cursor indicator

    let p = Paragraph::new(display_text)
        .block(block)
        .alignment(Alignment::Left)
        .style(theme::lyric_plain());

    // Clear the background so lyrics don't bleed through
    f.render_widget(Clear, popup_area);
    f.render_widget(p, popup_area);
}
