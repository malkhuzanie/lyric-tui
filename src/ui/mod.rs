/// UI module — layout orchestration.
///
/// This module is the only place that defines the layout geometry.
/// It delegates all rendering to the three widget sub-modules:
///
///   ┌─ header  ── 3 rows  ─ track info ─────────────────────────────┐
///   │                                                               │
///   ├─ lyrics  ── flex    ─ lyric lines (the primary viewport) ─────┤
///   │                                                               │
///   └─ footer  ── 3 rows  ─ progress gauge ────────────────────────-┘
///
/// Nothing else belongs here.  Colours live in `theme`, widget logic in
/// the respective sub-modules.

pub mod footer;
pub mod header;
pub mod lyrics;
pub mod theme;
pub mod help;
pub mod search;          
pub mod select_player;

use crate::app::{App, AppMode};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    widgets::{Block, Borders, Clear, Paragraph},
    style::{Style, Color},
    text::{Line, Span},
    Frame,
};

/// Helper to render popups centered
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Entry point called from the main render loop.
pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // header
            Constraint::Min(0),    // lyrics (grows to fill)
            Constraint::Length(3), // footer / progress
        ])
        .split(f.size());

    header::render(f, app, chunks[0]);
    lyrics::render(f, app, chunks[1]);
    footer::render(f, app, chunks[2]);

    match app.mode() {
        AppMode::Normal => {}
        AppMode::Help => {
            let area = centered_rect(40, 50, f.size()); 
            f.render_widget(Clear, area); 
            help::render(f, area);
        }
        AppMode::Search => {
            let area = centered_rect(60, 20, f.size());
            search::render(f, app, area);
        }
        AppMode::SelectPlayer => {
            let area = centered_rect(50, 40, f.size());
            select_player::render(f, app, area);
        }
    }
}
