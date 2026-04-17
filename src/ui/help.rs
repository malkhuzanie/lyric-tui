use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    style::Modifier,
    text::Span,
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};

use super::theme;

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

pub fn render(f: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" Keyboard Controls ")
        .borders(Borders::ALL)
        .border_style(theme::border_focus());

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let mut rows = Vec::new();
    let mut total_lines = 0;

    // 3. Dynamically build the table rows with headers
    for section in HELP_SECTIONS {
        // Add the Category Header row (Styled as Ivory + Underlined)
        rows.push(
            Row::new(vec![
                Cell::from(Span::styled(
                    section.title,
                    theme::track_title().add_modifier(Modifier::UNDERLINED),
                )),
                Cell::from(""), // Empty second column
            ])
            .bottom_margin(1), // Adds a blank line under the header
        );
        total_lines += 2;

        // Add the actual keybindings with a slight indent
        for (key, desc) in section.bindings {
            rows.push(Row::new(vec![
                Cell::from(Span::styled(format!("  {}", key), theme::hint_key())),
                Cell::from(Span::styled(*desc, theme::lyric_plain())),
            ]));
            total_lines += 1;
        }

        // Add a blank spacer row after each section
        rows.push(Row::new(vec![Cell::from(""), Cell::from("")]));
        total_lines += 1;
    }

    // 4. Calculate dimensions for Flex::Center
    let table_height = total_lines as u16;
    let col1_width = 20; // fits "Navigation & View" 
    let col2_width = 40; // fits "Toggle lyric provider (Lrclib/Genius)" 
    let spacing = 2;
    let table_width = col1_width + spacing + col2_width; // Total: 62

    let widths = [Constraint::Length(col1_width), Constraint::Length(col2_width)];

    let vertical_layout = Layout::vertical([Constraint::Length(table_height)])
        .flex(Flex::Center)
        .split(inner_area);

    let centered_area = Layout::horizontal([Constraint::Length(table_width)])
        .flex(Flex::Center)
        .split(vertical_layout[0])[0];

    // 5. Render the final table
    let table = Table::new(rows, widths).column_spacing(2);

    f.render_widget(table, centered_area);
}