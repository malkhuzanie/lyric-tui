use crate::app::{AppEvent, TrackInfo};
use super::common::clean_title;
use log::{error, info};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use regex::Regex;

const POLL_INTERVAL: Duration = Duration::from_millis(1000);

pub fn start(target_player: Arc<RwLock<Option<String>>>, tx: Sender<AppEvent>) {
    thread::spawn(move || {
        let mut tracker = super::common::TimelineTracker::new();

        loop {
            // Using mediaremote-rs or now-playing equivalent
            #[cfg(target_os = "macos")]
            {
                // Due to standard mediaremote-rs api, attempt to fetch global now playing state
                // This connects to the Private MRMediaRemoteGetNowPlayingInfo framework
                if let Ok(Some(info)) = mediaremote_rs::get_now_playing() {
                    let raw_title = info.title.unwrap_or_default();
                    let raw_artist = info.artist.unwrap_or_default();
                    let album = info.album.unwrap_or_default();

                    let title = clean_title(&raw_title);
                    let mut artist = clean_title(&raw_artist);

                    if artist.is_empty() || artist == title {
                        let parts: Vec<&str> = album.split(" - ").collect();
                        if parts.len() > 1 {
                            artist = parts[0].trim().to_string();
                        }
                    }

                    if !artist.is_empty() && !title.is_empty() {
                        let new_track = TrackInfo {
                            artist: artist.clone(),
                            title: title.clone(),
                            album,
                            length: info.duration.map(Duration::from_secs_f64),
                        };

                        let current_raw = Duration::from_secs_f64(info.elapsed_time.unwrap_or(0.0));
                        if tracker.process_track_change(new_track.clone(), current_raw) {
                            info!("Track changed [RAW MPIS]: artist: {:?}, title: {:?}", raw_artist, raw_title);
                            info!("Track changed [CLEANED]: {:?}", new_track);
                            let _ = tx.blocking_send(AppEvent::TrackChanged(new_track));
                        }
                    }

                    if let Some(raw_pos) = info.elapsed_time {
                        let pos = tracker.calculate_adjusted_position(Duration::from_secs_f64(raw_pos));
                        let _ = tx.blocking_send(AppEvent::PositionUpdated(pos));
                    }
                }
            }

            thread::sleep(POLL_INTERVAL);
        }
    });
}
