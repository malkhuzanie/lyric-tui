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
    layout::{Constraint, Direction, Layout},
    Frame,
};

pub fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();
    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line.push_str(word);
        } else if current_line.len() + 1 + word.len() > width {
            lines.push(current_line);
            current_line = word.to_string();
        } else {
            current_line.push(' ');
            current_line.push_str(word);
        }
    }
    if !current_line.is_empty() {
        lines.push(current_line);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

/// Entry point called from the main render loop.
pub fn render(f: &mut Frame, app: &App) {
    if app.config.view.full_screen {
        lyrics::render(f, app, f.size());
    } else {
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
    }

    match app.mode() {
        AppMode::Normal => {}
        AppMode::Help => {
            help::render(f, f.size());
        }
        AppMode::Search => {
            search::render(f, app, f.size());
        }
        AppMode::SelectPlayer => {
            select_player::render(f, app, f.size());
        }
    }
}
