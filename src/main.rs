mod media_monitor;
mod app;
mod events;
mod providers;
mod ui;
mod cli;

use app::{App, AppCommand, AppEvent, AppMode, LyricLine, Config, Provider, LyricAlignment};
use providers::{LyricProvider, genius::GeniusProvider, lrclib::LrclibProvider};
use crossterm::{
    event::{Event, EventStream},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::StreamExt;
use ratatui::{backend::CrosstermBackend, Terminal};
use simplelog::{WriteLogger, LevelFilter, Config as LogConfig};
use std::env;
use std::fs::File;
use std::io;
use std::time::Duration;
use std::sync::{Arc, RwLock};
use directories::ProjectDirs;
use tokio::sync::mpsc;
use tokio::time::interval;

use cli::Cli;
use clap::Parser;

// ── Config I/O ────────────────────────────────────────────────────────────────

fn load_config() -> Config {
    let mut config = Config::default();

    if let Some(dirs) = ProjectDirs::from("com", "tomato", "lyric-tui") {
        let config_dir = dirs.config_dir();
        std::fs::create_dir_all(config_dir).ok();
        let config_file = config_dir.join("config.toml");

        if config_file.exists() {
            log::info!("Found config file at {}", config_file.display());
            match std::fs::read_to_string(&config_file) {
                Ok(content) => match toml::from_str(&content) {
                    Ok(parsed) => {
                        log::info!("Successfully loaded user configuration.");
                        config = parsed;
                    }
                    Err(e) => {
                        log::error!("Failed to parse config.toml: {}. Falling back to default settings.", e);
                    }
                },
                Err(e) => log::error!("Could not read config file: {}", e),
            }
        } else {
            log::info!("No config file found. Generating default config at {}", config_file.display());
            if let Ok(toml) = toml::to_string(&config) {
                if let Err(e) = std::fs::write(&config_file, toml) {
                    log::error!("Failed to write default config file: {}", e);
                }
            }
        }
    } else {
        log::warn!("Could not determine OS configuration directory.");
    }

    config
}

fn save_config(config: &Config) {
    if let Some(dirs) = ProjectDirs::from("com", "tomato", "lyric-tui") {
        let path = dirs.config_dir().join("config.toml");
        log::info!("Saving configuration state to {}", path.display());
        
        match toml::to_string(config) {
            Ok(serialized) => {
                if let Err(e) = std::fs::write(&path, serialized) {
                    log::error!("Failed to save configuration to disk: {}", e);
                } else {
                    log::info!("Configuration saved successfully.");
                }
            }
            Err(e) => log::error!("Failed to serialize configuration state: {}", e),
        }
    }
}

// ── Lyric fetching ────────────────────────────────────────────────────────────

pub async fn fetch_lyrics(
    provider: Provider,
    artist: &str,
    title: &str,
    force_refresh: bool,
    timeout_secs: u64,
) -> anyhow::Result<Vec<LyricLine>> {
    log::info!(
        "Fetching lyrics from {:?} for artist: {:?}, title: {:?}",
        provider, artist, title
    );
    match provider {
        Provider::Lrclib => LrclibProvider::new(timeout_secs).fetch(artist, title, force_refresh).await,
        Provider::Genius => GeniusProvider::new(timeout_secs).fetch(artist, title, force_refresh).await,
    }
}

// ── Command processor ─────────────────────────────────────────────────────────
//
// This is the *only* place that calls `tokio::spawn` or writes to
// `shared_target_player`. The event handler returns an `AppCommand`; we
// execute it here so that all async orchestration stays in one callsite.

fn process_command(
    cmd: AppCommand,
    app: &App,
    tx: &mpsc::Sender<AppEvent>,
    shared_target_player: &Arc<RwLock<Option<String>>>,
) -> bool {
    match cmd {
        AppCommand::Quit => {
            return true;
        }

        AppCommand::FetchLyrics { artist, title, force_refresh } => {
            let provider = app.config.view.default_provider.clone();
            let timeout_secs = app.config.core.network_timeout_secs; 
            let tx_clone = tx.clone();
            tokio::spawn(async move {
                let fetched = fetch_lyrics(provider, &artist, &title, force_refresh, timeout_secs)
                    .await
                    .unwrap_or_else(|e| {
                        log::error!("Failed to fetch lyrics (timeout: {}s): {}", timeout_secs, e);
                        
                        let error_msg = format!("Network error (gave up after {}s). Press 'r' to retry.", timeout_secs);
                        vec![LyricLine { start_time: None, text: error_msg }]
                    });
                let _ = tx_clone.send(AppEvent::LyricsFetched(fetched)).await;
            });
        }

        AppCommand::SwitchPlayer(name) => {
            match shared_target_player.write() {
                Ok(mut guard) => *guard = Some(name),
                Err(poisoned) => {
                    log::error!("target_player mutex poisoned — recovering");
                    *poisoned.into_inner() = Some(name);
                }
            }
        }

        AppCommand::ClearPlayerTarget => {
            match shared_target_player.write() {
                Ok(mut guard) => *guard = None,
                Err(poisoned) => {
                    log::error!("target_player mutex poisoned — recovering");
                    *poisoned.into_inner() = None;
                }
            }
        }
    }

    false
}

// ── Entry point ───────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ── Logging ──────────────────────────────────────────────────────────────
    // Non-fatal: if we can't create the log file, continue without logging.
    match File::create("lyric-tui.log") {
        Ok(log_file) => {
            WriteLogger::init(LevelFilter::Debug, LogConfig::default(), log_file).ok();
        }
        Err(e) => {
            eprintln!("Warning: could not create lyric-tui.log: {e}");
        }
    }
    log::info!("=== Starting lyric-tui ===");

    // ── Terminal setup ───────────────────────────────────────────────────────
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // ── CLI args ─────────────────────────────────────────────────────────────
    // let args: Vec<String> = env::args().collect();
    // let target_player = args.get(1).cloned();
    let cli_args = Cli::parse();
    let target_player = cli_args.target_player;

    log::info!("Target player from args: {:?}", target_player);

    let shared_target_player = Arc::new(RwLock::new(target_player));

    // ── Async channels ───────────────────────────────────────────────────────
    let (tx, mut rx) = mpsc::channel::<AppEvent>(64);

    // ── Media monitor thread (OS-specific) ───────────────────────────────────
    media_monitor::start(shared_target_player.clone(), tx.clone());

    // ── Keyboard input task ──────────────────────────────────────────────────
    let tx_input = tx.clone();
    tokio::spawn(async move {
        let mut reader = EventStream::new();
        while let Some(Ok(event)) = reader.next().await {
            if let Event::Key(key) = event {
                let _ = tx_input.send(AppEvent::Input(key)).await;
            }
        }
    });

    // ── UI ticker — 60 fps ───────────────────────────────────────────────────
    let tx_tick = tx.clone();
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_millis(16));
        loop {
            ticker.tick().await;
            let _ = tx_tick.send(AppEvent::Tick).await;
        }
    });

    // ── App state ────────────────────────────────────────────────────────────
    let mut app = App::new();
    app.config = load_config();
    ui::theme::init_from_config(&app.config.theme);
    app.set_status_lyric("Waiting for track data…");

    // ── Main event loop ──────────────────────────────────────────────────────
    //
    // Dirty flag: only redraw when state actually changed.  The Tick event
    // marks the frame dirty so animations (progress bar, auto-scroll) stay
    // smooth without redrawing on every channel receive.
    let mut dirty = true;

    loop {
        // Keep viewport height in sync with terminal size.
        // Header (3 rows) + footer (3 rows) = 6 rows of chrome.
        let size = terminal.size()?;
        let new_height = size.height.saturating_sub(6);
        if new_height != app.view.viewport_height {
            app.view.viewport_height = new_height;
            app.recalculate_scroll();
            dirty = true;
        }

        if dirty {
            terminal.draw(|f| ui::render(f, &app))?;
            dirty = false;
        }

        if let Some(event) = rx.recv().await {
            match event {
                // ── Input ─────────────────────────────────────────────────────
                AppEvent::Input(key) => {
                    let (should_quit, maybe_cmd) = events::handle_key_event(key, &mut app);
                    dirty = true;

                    if let Some(cmd) = maybe_cmd {
                        // `AppCommand::Quit` can come from either the quit key
                        // or Ctrl-C; in both cases `should_quit` is also true,
                        // but we save config before breaking.
                        let is_quit = matches!(cmd, AppCommand::Quit);
                        process_command(cmd, &app, &tx, &shared_target_player);
                        if is_quit || should_quit {
                            break;
                        }
                    } else if should_quit {
                        break;
                    }
                }

                // ── Track changed ─────────────────────────────────────────────
                AppEvent::TrackChanged(track) => {
                    log::info!("Track changed: {} – {}", track.artist, track.title);
                    let artist = track.artist.clone();
                    let title = track.title.clone();

                    app.playback.track = Some(track);
                    app.set_status_lyric("Fetching lyrics…");
                    app.view.auto_scroll = true;

                    process_command(
                        AppCommand::FetchLyrics { artist, title, force_refresh: false },
                        &app,
                        &tx,
                        &shared_target_player,
                    );
                    dirty = true;
                }

                // ── Position update ───────────────────────────────────────────
                AppEvent::PositionUpdated(pos) => {
                    app.update_position(pos);
                    dirty = true;
                }

                // ── Lyrics arrived ────────────────────────────────────────────
                AppEvent::LyricsFetched(lyric_lines) => {
                    log::info!("Lyrics updated ({} lines)", lyric_lines.len());
                    app.lyrics = lyric_lines;
                    app.recalculate_scroll();
                    dirty = true;
                }

                // ── Player list refresh ───────────────────────────────────────
                AppEvent::PlayersDiscovered(players) => {
                    // The monitor already deduplicates; only update if changed.
                    if players != app.players.players {
                        if app.players.selected_idx >= players.len() {
                            app.players.selected_idx = players.len().saturating_sub(1);
                        }
                        app.players.players = players;
                        dirty = true;
                    }
                }

                // ── Tick — just mark the frame dirty for smooth animation ─────
                AppEvent::Tick => {
                    dirty = true;
                }
            }
        }
    }

    // ── Shutdown ──────────────────────────────────────────────────────────────
    log::info!("Shutting down — saving config");
    save_config(&app.config);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
