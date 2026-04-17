/// Lyrics pane widget.
///
/// Renders the lyric lines with a clear three-state colour model:
///
///   PAST    — already sung, dimmed to slate so the eye skips over them
///   ACTIVE  — the current line, gold + bold, with a ▶ playhead marker
///   FUTURE  — upcoming lines, warm linen
///
/// For unsynced (plain) lyrics every line uses the `lyric_plain` style and
/// the playhead / timestamp chrome is hidden entirely.
///
/// The panel title doubles as a minimal status bar, keeping the hint text
/// tucked into the border rather than stealing a dedicated row.
use crate::app::App;
use ratatui::{
    layout::{Alignment, Rect},
    text::{Line, Span},
    widgets::{block::Title, Block, BorderType, Borders, Paragraph, Wrap},
    Frame,
};

use super::theme;

// Glyphs — defined once, easy to swap
const PLAYHEAD: &str = "▶ ";
const PLAYHEAD_SPACE: &str = "  "; // same width when no playhead
const SEPARATOR: &str = "·";

/// Renders the lyrics pane into `area`.
pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let has_sync = app.lyrics.iter().any(|l| l.start_time.is_some());
    let lines = build_lyric_lines(app, has_sync, &app.config.view.alignment);

    let block = build_block(app);

    let alignment = match app.config.view.alignment {
        crate::app::LyricAlignment::Left => Alignment::Left,
        crate::app::LyricAlignment::Center => Alignment::Center,
    };

    let paragraph = Paragraph::new(lines)
        .alignment(alignment)
        .wrap(Wrap { trim: false })
        .scroll((app.view.scroll, 0))
        .block(block);

    f.render_widget(paragraph, area);
}

// ── Line builder ─────────────────────────────────────────────────────────────

fn build_lyric_lines<'a>(
    app: &'a App,
    has_sync: bool,
    alignment: &crate::app::LyricAlignment,
) -> Vec<Line<'a>> {
    let show_chrome = matches!(alignment, crate::app::LyricAlignment::Left);

    app.lyrics
        .iter()
        .enumerate()
        .map(|(i, line)| {
            let is_active = has_sync && i == app.playback.active_line;
            let is_past = has_sync && i < app.playback.active_line;

            let text_style = if !has_sync {
                theme::lyric_plain()
            } else if is_active {
                theme::lyric_active()
            } else if is_past {
                theme::lyric_past()
            } else {
                theme::lyric_future()
            };

            let mut spans: Vec<Span<'a>> = Vec::with_capacity(4);

            // Timestamp column (only when synced AND left-aligned)
            if has_sync && show_chrome {
                let ts = match line.start_time {
                    Some(t) => {
                        let secs = t.as_secs();
                        format!("{:02}:{:02} {} ", secs / 60, secs % 60, SEPARATOR)
                    }
                    None => "       ".to_string(), // align with timestamps
                };
                spans.push(Span::styled(ts, theme::timestamp()));
            }

            // Playhead column (only when synced AND left-aligned)
            if has_sync && show_chrome {
                if is_active {
                    spans.push(Span::styled(app.config.view.playhead_symbol.clone(), theme::playhead()));
                } else {
                    spans.push(Span::styled(PLAYHEAD_SPACE, theme::timestamp()));
                }
            }

            // Lyric text
            spans.push(Span::styled(line.text.clone(), text_style));

            Line::from(spans)
        })
        .collect()
}

// ── Block / border builder ────────────────────────────────────────────────────

fn build_block(app: &App) -> Block<'static> {
    let left_title = Span::styled(" LYRICS ", theme::hint_desc());

    let autoscroll_span = if app.view.auto_scroll {
        Span::styled("ON ", theme::autoscroll_on())
    } else {
        Span::styled("OFF ", theme::autoscroll_off())
    };

    // Build the right-side hint line: alternating gold keys and slate descriptions.
    let mut hint_spans: Vec<Span<'static>> = Vec::new();
    for (key, desc) in &[("q", "quit"), ("r", "reload"), ("p", "provider"), ("c", "center"), ("j/k", "scroll"), ("a", "auto:")] {
        hint_spans.push(Span::styled(format!(" {key}"), theme::hint_key()));
        hint_spans.push(Span::styled(format!(" {desc}"), theme::hint_desc()));
        hint_spans.push(Span::styled("  ", theme::hint_desc()));
    }
    hint_spans.push(autoscroll_span);

    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(theme::border_focus())
        .title(Title::from(left_title).alignment(Alignment::Left))
        .title(Title::from(Line::from(hint_spans)).alignment(Alignment::Right))
}
