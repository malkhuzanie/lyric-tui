use crate::app::{AppEvent, TrackInfo};
use super::common::clean_title;
use log::{error, info};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use regex::Regex;
use windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager;

const POLL_INTERVAL: Duration = Duration::from_millis(1000); // 1 second for windows to minimize overhead

pub fn start(target_player: Arc<RwLock<Option<String>>>, tx: Sender<AppEvent>) {
    thread::spawn(move || {
        // Initialize COM for the background thread
        let com_result = unsafe {
            windows::Win32::System::Com::CoInitializeEx(
                None,
                windows::Win32::System::Com::COINIT_APARTMENTTHREADED,
            )
        };

        if !com_result.is_ok() {
            error!("Failed to initialize COM: {:?}", com_result);
            return;
        }
            
        let manager_result = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
            .and_then(|op| op.get());

        let manager = match manager_result {
            Ok(m) => m,
            Err(e) => {
                error!("Failed to initialize Windows Media Controls: {}", e);
                return;
            }
        };

        let mut current_track: Option<TrackInfo> = None;

        loop {
            let target = target_player.read().unwrap().clone();
            let session_result = match target {
                Some(ref target_id) => {
                    let mut found_session = Err(windows::core::Error::from(windows::core::HRESULT(0)));
                    if let Ok(sessions) = manager.GetSessions() {
                        for s in sessions {
                            if let Ok(app_id) = s.SourceAppUserModelId() {
                                if app_id.to_string_lossy() == *target_id {
                                    found_session = Ok(s);
                                    break;
                                }
                            }
                        }
                    }
                    found_session
                },
                None => manager.GetCurrentSession()
            };
            
            if let Ok(session) = session_result {
                // To fetch players
                let mut available_players = vec![];
                if let Ok(sessions) = manager.GetSessions() {
                    for s in sessions {
                        if let Ok(app_id) = s.SourceAppUserModelId() {
                            available_players.push(app_id.to_string_lossy());
                        }
                    }
                }
                let _ = tx.blocking_send(AppEvent::PlayersDiscovered(available_players));

                if let Ok(timeline) = session.GetTimelineProperties() {
                    let pos = timeline.Position().unwrap_or_default();
                    let pos_duration = Duration::from_nanos((pos.Duration as u64) * 100);
                    let _ = tx.blocking_send(AppEvent::PositionUpdated(pos_duration));
                }

                if let Ok(props_op) = session.TryGetMediaPropertiesAsync() {
                    if let Ok(props) = props_op.get() {
                        let raw_title = props.Title().unwrap_or_default().to_string_lossy();
                        let raw_artist = props.Artist().unwrap_or_default().to_string_lossy();
                        let album = props.AlbumTitle().unwrap_or_default().to_string_lossy();

                        let title = clean_title(&raw_title);
                        let mut artist = clean_title(&raw_artist);

                        // Broken metadata fallback
                        if artist.is_empty() || artist == title {
                            let parts: Vec<&str> = album.split(" - ").collect();
                            if parts.len() > 1 {
                                artist = parts[0].trim().to_string();
                            }
                        }
                    
                        let length = if let Ok(timeline) = session.GetTimelineProperties() {
                            let end = timeline.EndTime().unwrap_or_default();
                            Some(Duration::from_nanos((end.Duration as u64) * 100))
                        } else {
                            None
                        };

                        if !artist.is_empty() && !title.is_empty() {
                            let new_track = TrackInfo {
                                artist: artist.clone(),
                                title: title.clone(),
                                album,
                                length: length,
                            };

                            if Some(&new_track) != current_track.as_ref() {
                                info!("Track changed [RAW MPIS]: artist: {:?}, title: {:?}", raw_artist, raw_title);
                                info!("Track changed [CLEANED]: {:?}", new_track);
                                current_track = Some(new_track.clone());
                                let _ = tx.blocking_send(AppEvent::TrackChanged(new_track));
                            }
                        }
                    }
                }
            }

            thread::sleep(POLL_INTERVAL);
        }
    });
}
