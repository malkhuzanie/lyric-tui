use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    style::Modifier,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Cell, Clear, Padding, Row, Table},
    Frame,
};

use super::{theme, wrap_text};

// 1. Define a structure for our categories
struct HelpSection<'a> {
    title: &'a str,
    bindings: &'a [(&'a str, &'a str)],
}

// 2. Group the bindings logically
const HELP_SECTIONS: &[HelpSection] = &[
    HelpSection {
        title: "Navigation & View",
        bindings: &[
            ("j / Down", "Scroll lyrics down"),
            ("k / Up", "Scroll lyrics up"),
            ("a", "Toggle auto-scroll"),
            ("c", "Toggle text alignment (Left/Center)"),
            ("f", "Toggle full-screen mode"),
        ],
    },
    HelpSection {
        title: "Data & Providers",
        bindings: &[
            ("s", "Manual search (Artist - Title)"),
            ("p", "Toggle lyric provider (Lrclib/Genius)"),
            ("r", "Force network reload"),
        ],
    },
    HelpSection {
        title: "Application",
        bindings: &[
            ("l", "Switch active media player"),
            ("? / h", "Close this help menu"),
            ("q / Esc", "Quit the application"),
        ],
    },
];


pub fn render(f: &mut Frame, screen_area: Rect) {
    let col1_width = 20; // fits longest key binding with indent
    let max_col2_width = 45; // fits longest description un-wrapped
    let spacing = 2;
    let table_borders = 2;
    let horizontal_padding = 4; // 2 left, 2 right
    let vertical_padding = 2;   // 1 top, 1 bottom

    // Total preferred width
    let preferred_table_width = col1_width + spacing + max_col2_width;
    let preferred_popup_width = preferred_table_width + table_borders + horizontal_padding;

    // Constrain width by actual screen real estate (leave some margin)
    let actual_popup_width = preferred_popup_width.min(screen_area.width.saturating_sub(4));
    let actual_table_width = actual_popup_width.saturating_sub(table_borders + horizontal_padding);

    // Calculate the interactive col2 width based on available space
    let actual_col2_width = actual_table_width.saturating_sub(col1_width + spacing).max(10);

    let mut rows = Vec::new();
    let mut total_table_height = 0;

    for section in HELP_SECTIONS {
        rows.push(
            Row::new(vec![
                Cell::from(Span::styled(
                    section.title,
                    theme::track_title().add_modifier(Modifier::UNDERLINED),
                )),
                Cell::from(""), // Empty second column
            ])
            .bottom_margin(1),
        );
        total_table_height += 2;

        for (key, desc) in section.bindings {
            let wrapped_desc = wrap_text(desc, actual_col2_width as usize);
            let row_height = wrapped_desc.len().max(1) as u16;

            let desc_lines: Vec<Line> = wrapped_desc
                .into_iter()
                .map(|line| Line::from(Span::styled(line, theme::lyric_plain())))
                .collect();

            rows.push(
                Row::new(vec![
                    Cell::from(Span::styled(format!("  {}", key), theme::hint_key())),
                    Cell::from(Text::from(desc_lines)),
                ])
                .height(row_height),
            );

            total_table_height += row_height;
        }

        // Blank spacer row after each section
        rows.push(Row::new(vec![Cell::from(""), Cell::from("")]).height(1));
        total_table_height += 1;
    }

    // Popup height is table height + borders, constrained by screen height
    let preferred_popup_height = total_table_height + table_borders + vertical_padding;
    let actual_popup_height = preferred_popup_height.min(screen_area.height.saturating_sub(2));

    let vertical_layout = Layout::vertical([Constraint::Length(actual_popup_height)])
        .flex(Flex::Center)
        .split(screen_area);

    let popup_area = Layout::horizontal([Constraint::Length(actual_popup_width)])
        .flex(Flex::Center)
        .split(vertical_layout[0])[0];

    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(" Keyboard Controls ")
        .borders(Borders::ALL)
        .border_style(theme::border_focus())
        .padding(Padding::symmetric(2, 1));

    let inner_area = block.inner(popup_area);
    f.render_widget(block, popup_area);

    let widths = [Constraint::Length(col1_width), Constraint::Length(actual_col2_width)];
    let table = Table::new(rows, widths).column_spacing(spacing);

    f.render_widget(table, inner_area);
}