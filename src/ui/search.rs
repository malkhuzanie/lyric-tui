use ratatui::{
    layout::{Alignment, Rect},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::App;
use super::theme;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Manual Search (Artist - Title) ")
        .borders(Borders::ALL)
        .border_style(theme::border_focus());

    let mut display_text = app.search.input.clone();
    display_text.push('█'); // simple cursor indicator

    let p = Paragraph::new(display_text)
        .block(block)
        .alignment(Alignment::Left)
        .style(theme::lyric_plain());

    // Clear the background so lyrics don't bleed through
    f.render_widget(Clear, area);
    f.render_widget(p, area);
}
