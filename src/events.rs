/// Input event handler.
///
/// ## Design contract
///
/// `handle_key_event` is a **pure state-mutation function** — it mutates `App`
/// and returns one of three outcomes:
///
/// | Return value              | Meaning                                      |
/// |---------------------------|----------------------------------------------|
/// | `(false, None)`           | Normal keypress; UI will re-render.          |
/// | `(false, Some(command))`  | State updated AND a side-effect is needed.   |
/// | `(true,  _)`              | Quit requested; main loop should break.      |
///
/// **It never calls `tokio::spawn`, never clones a `Sender`, and never touches
/// the `Arc<Mutex<…>>` directly.** All of that lives in `main.rs` which
/// processes the returned `AppCommand`.  This makes the handler trivially
/// unit-testable with no async runtime.
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, KeyEventKind};

use crate::app::{App, AppCommand, AppMode, LyricAlignment, LyricLine, Provider, TrackInfo};

/// Handle one key event.  Returns `(should_quit, optional_command)`.
pub fn handle_key_event(key: KeyEvent, app: &mut App) -> (bool, Option<AppCommand>) {
    if key.kind != KeyEventKind::Press {
        return (false, None);
    }

    // Ctrl-C is always a hard quit regardless of mode.
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return (true, Some(AppCommand::Quit));
    }

    match app.mode() {
        AppMode::Normal => handle_normal(key, app),
        AppMode::Help => handle_help(key, app),
        AppMode::Search => handle_search(key, app),
        AppMode::SelectPlayer => handle_select_player(key, app),
    }
}

// ── Mode handlers ─────────────────────────────────────────────────────────────

fn handle_normal(key: KeyEvent, app: &mut App) -> (bool, Option<AppCommand>) {
    match key.code {
        // ── Quit ──────────────────────────────────────────────────────────────
        KeyCode::Char('q') | KeyCode::Esc => return (true, Some(AppCommand::Quit)),

        // ── Scroll ───────────────────────────────────────────────────────────
        KeyCode::Down | KeyCode::Char('j') => app.scroll_down(),
        KeyCode::Up | KeyCode::Char('k') => app.scroll_up(),

        // ── Auto-scroll toggle ────────────────────────────────────────────────
        KeyCode::Char('a') => app.toggle_auto_scroll(),

        // ── Full-screen toggle ────────────────────────────────────────────────
        KeyCode::Char('f') => {
            log::info!("Toggled full-screen mode: {}", !app.config.view.full_screen);
            app.config.view.full_screen = !app.config.view.full_screen;
        }

        // ── Alignment toggle ──────────────────────────────────────────────────
        KeyCode::Char('c') => {
            app.config.view.alignment = match app.config.view.alignment {
                LyricAlignment::Left => LyricAlignment::Center,
                LyricAlignment::Center => LyricAlignment::Left,
            };
        }

        // ── Provider toggle ───────────────────────────────────────────────────
        KeyCode::Char('p') => {
            app.config.view.default_provider = match app.config.view.default_provider {
                Provider::Lrclib => Provider::Genius,
                Provider::Genius => Provider::Lrclib,
            };
            log::info!("Switched provider to {:?}", app.config.view.default_provider);

            if let Some(track) = &app.playback.track {
                let artist = track.artist.clone();
                let title = track.title.clone();
                app.set_status_lyric(format!(
                    "Switching to {:?} provider…",
                    app.config.view.default_provider
                ));
                return (
                    false,
                    Some(AppCommand::FetchLyrics {
                        artist,
                        title,
                        force_refresh: false,
                        alternate_artist: None,
                        alternate_title: None,
                    }),
                );
            }
        }

        // ── Force reload ──────────────────────────────────────────────────────
        KeyCode::Char('r') => {
            log::info!("Manual refresh triggered");
            if let Some(track) = &app.playback.track {
                let artist = track.artist.clone();
                let title = track.title.clone();
                app.set_status_lyric("Reloading lyrics from network…");
                return (
                    false,
                    Some(AppCommand::FetchLyrics {
                        artist,
                        title,
                        force_refresh: true,
                        alternate_artist: None,
                        alternate_title: None,
                    }),
                );
            }
        }

        // ── Open search popup ─────────────────────────────────────────────────
        KeyCode::Char('s') => {
            app.search.input.clear();
            app.set_mode(AppMode::Search);
        }

        // ── Open help popup ───────────────────────────────────────────────────
        KeyCode::Char('?') | KeyCode::Char('h') => {
            app.set_mode(AppMode::Help);
        }

        // ── Open player selector popup ────────────────────────────────────────
        KeyCode::Char('l') => {
            app.players.selected_idx = 0;
            app.set_mode(AppMode::SelectPlayer);
        }

        _ => {}
    }

    (false, None)
}

fn handle_help(key: KeyEvent, app: &mut App) -> (bool, Option<AppCommand>) {
    match key.code {
        KeyCode::Char('q')
        | KeyCode::Esc
        | KeyCode::Char('h')
        | KeyCode::Char('?')
        | KeyCode::Enter => {
            app.set_mode(AppMode::Normal);
        }
        _ => {}
    }
    (false, None)
}

fn handle_search(key: KeyEvent, app: &mut App) -> (bool, Option<AppCommand>) {
    match key.code {
        KeyCode::Esc => {
            app.set_mode(AppMode::Normal);
        }

        KeyCode::Backspace => {
            app.search.input.pop();
        }

        KeyCode::Char(c) => {
            app.search.input.push(c);
        }

        KeyCode::Enter => {
            app.set_mode(AppMode::Normal);
            let query = app.search.input.trim().to_string();

            if !query.is_empty() {
                log::info!("Manual search: {}", query);

                // Use " - " (spaced hyphen) as the delimiter to correctly handle
                // artist names that themselves contain hyphens (e.g., "AC-DC").
                let (artist, title) = query
                    .split_once(" - ")
                    .map(|(a, t)| (a.trim().to_string(), t.trim().to_string()))
                    .unwrap_or_else(|| (query.clone(), query.clone()));

                // Update playback track immediately so the header shows the
                // searched-for song while lyrics are loading.
                // Preserve all metadata from the currently playing track (if any)
                // to ensure cache consistency: lyrics are cached under the player's
                // canonical metadata, allowing cache hits even with manual search.
                if let Some(current_track) = &app.playback.track {
                    // Preserve the player's original metadata to maintain consistent
                    // cache keys. This allows manual search results to be reused when
                    // the same track is later detected by the media monitor.
                    app.playback.track = Some(TrackInfo {
                        artist: current_track.artist.clone(),
                        title: current_track.title.clone(),
                        album: current_track.album.clone(),
                        length: current_track.length,
                    });
                } else {
                    // No track currently playing — use search query metadata.
                    app.playback.track = Some(TrackInfo {
                        artist: artist.clone(),
                        title: title.clone(),
                        album: String::new(),
                        length: None,
                    });
                }
                app.set_status_lyric(format!("Searching for {} — {}…", artist, title));
                app.view.auto_scroll = false;

                // When a manual search finds lyrics, we want to cache them under BOTH
                // the search query metadata AND the player's original metadata (if available).
                // This way, future automatic playback of the same song will find the cache
                // using the player's metadata, avoiding a redundant network request.
                let (alternate_artist, alternate_title) = if let Some(current_track) = &app.playback.track {
                    // We're searching for a different song than what's playing, or the same song
                    // with cleaned-up metadata. Cache under both the search query and original.
                    (Some(current_track.artist.clone()), Some(current_track.title.clone()))
                } else {
                    (None, None)
                };

                return (
                    false,
                    Some(AppCommand::FetchLyrics {
                        artist,
                        title,
                        force_refresh: false,
                        alternate_artist,
                        alternate_title,
                    }),
                );
            }
        }

        _ => {}
    }

    (false, None)
}

fn handle_select_player(key: KeyEvent, app: &mut App) -> (bool, Option<AppCommand>) {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.set_mode(AppMode::Normal);
        }

        KeyCode::Down | KeyCode::Char('j') => {
            if !app.players.players.is_empty() {
                app.players.selected_idx =
                    (app.players.selected_idx + 1) % app.players.players.len();
            }
        }

        KeyCode::Up | KeyCode::Char('k') => {
            if !app.players.players.is_empty() {
                app.players.selected_idx = app
                    .players
                    .selected_idx
                    .checked_sub(1)
                    .unwrap_or(app.players.players.len() - 1);
            }
        }

        KeyCode::Enter => {
            app.set_mode(AppMode::Normal);

            let cmd = if app.players.players.is_empty() {
                log::info!("Cleared target player");
                AppCommand::ClearPlayerTarget
            } else {
                let selected = app.players.players[app.players.selected_idx].clone();
                log::info!("Switched target player to: {}", selected);
                app.playback.track = None;
                app.set_status_lyric(format!("Waiting for {}…", selected));
                AppCommand::SwitchPlayer(selected)
            };

            return (false, Some(cmd));
        }

        _ => {}
    }

    (false, None)
}