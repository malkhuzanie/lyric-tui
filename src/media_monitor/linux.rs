/// MPRIS monitor — runs on a dedicated OS thread.
///
/// Polls the D-Bus session bus at ~5 Hz (every 200 ms) to:
///   1. Emit `PositionUpdated` events for smooth lyric auto-scrolling.
///   2. Emit `TrackChanged` events when the metadata changes.
///
/// A specific player identity can be targeted via `target_player`; otherwise
/// the active player (or the first available one) is used.
use crate::app::{AppEvent, TrackInfo};
use super::common::clean_title;
use log::{error, info};
use mpris::PlayerFinder;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use regex::Regex;

const POLL_INTERVAL: Duration = Duration::from_millis(200);

pub fn start(target_player: Arc<RwLock<(Option<String>, usize)>>, tx: Sender<AppEvent>) {
    thread::spawn(move || {
        let finder = match PlayerFinder::new() {
            Ok(f) => f,
            Err(e) => {
                error!("Failed to connect to D-Bus: {}", e);
                return;
            }
        };

        let mut tracker = super::common::TimelineTracker::new();
        let mut current_player_identity: Option<String> = None;
        let mut last_gen: usize = 0;

        loop {
            let (target, generation) = {
                let guard = target_player.read().unwrap();
                (guard.0.clone(), guard.1)
            };
            
            if generation != last_gen {
                last_gen = generation;
                tracker.reset_player();
            }

            if let Some(player) = find_player(&finder, target, &tx) {
                let identity = player.identity().to_string();
                if Some(&identity) != current_player_identity.as_ref() {
                    current_player_identity = Some(identity);
                    tracker.reset_player(); // Force track update on player switch
                }

                // Track-change detection — only emit when something differs.
                if let Ok(metadata) = player.get_metadata() {
                    let raw_artist = metadata.artists().unwrap_or_default().join(", ");
                    let raw_title = metadata.title().unwrap_or_default().to_string();
                    let title = clean_title(&raw_title);
                    let mut artist = clean_title(&raw_artist);
                    let album = metadata.album_name().unwrap_or_default().to_string();
                    let length = metadata.length();

                    // Heuristic fallback for broken metadata from some players (like roon-mpris handling missing tags)
                    // If artist is same as title, or missing, but album has "Artist - Album" format
                    if artist.is_empty() || artist == title {
                        let parts: Vec<&str> = album.split(" - ").collect();
                        if parts.len() > 1 {
                            artist = parts[0].trim().to_string();
                        }
                    }

                    let final_artist = if artist.is_empty() { "Unknown Artist".to_string() } else { artist };
                    if !title.is_empty() {
                        let new_track = TrackInfo { artist: final_artist, title: title.clone(), album: album.clone(), length };
                        let current_raw = player.get_position().unwrap_or(Duration::ZERO);

                        if tracker.process_track_change(new_track.clone(), current_raw) {
                            info!("Track changed [RAW MPIS]: artist: {:?}, title: {:?}, album: {:?}", raw_artist, raw_title, album);
                            info!("Track changed [CLEANED]: {:?}", new_track);
                            let _ = tx.blocking_send(AppEvent::TrackChanged(new_track));
                        }
                    }
                }

                // Position updates drive real-time lyric scrolling.
                // Moved after metadata polling to ensure offset is captured correctly upon track transition
                if let Ok(raw_pos) = player.get_position() {
                    let pos = tracker.calculate_adjusted_position(raw_pos);
                    let _ = tx.blocking_send(AppEvent::PositionUpdated(pos));
                }
            }

            thread::sleep(POLL_INTERVAL);
        }
    });
}

fn find_player<'a>(
    finder: &'a PlayerFinder,
    target: Option<String>,
    tx: &Sender<AppEvent>,
) -> Option<mpris::Player> {
    let mut available_players = vec![];
    if let Ok(players) = finder.find_all() {
        for p in players.iter() {
            available_players.push(p.identity().to_string());
        }
    }
    let _ = tx.blocking_send(AppEvent::PlayersDiscovered(available_players));

    match target {
        Some(name) => finder.find_by_name(&name).ok(),
        None => finder
            .find_active()
            .ok()
            .or_else(|| finder.find_all().ok().and_then(|mut players| players.pop())),
    }
}
