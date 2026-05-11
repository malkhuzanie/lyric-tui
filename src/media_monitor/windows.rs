use crate::app::{AppEvent, TrackInfo};
use super::common::clean_title;
use log::{error, info, debug, trace};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc::Sender;
use regex::Regex;
use windows::Media::Control::{GlobalSystemMediaTransportControlsSessionManager, GlobalSystemMediaTransportControlsSessionPlaybackStatus};

const POLL_INTERVAL: Duration = Duration::from_millis(1000); // 1 second for windows to minimize overhead

pub fn start(target_player: Arc<RwLock<(Option<String>, usize)>>, tx: Sender<AppEvent>) {
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

        let mut tracker = super::common::TimelineTracker::new();
        let mut last_raw_pos: i64 = 0;
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

                            let current_raw = if let Ok(timeline) = session.GetTimelineProperties() {
                                Duration::from_nanos((timeline.Position().unwrap_or_default().Duration as u64) * 100)
                            } else {
                                Duration::ZERO
                            };

                            if tracker.process_track_change(new_track.clone(), current_raw) {
                                info!("Track changed [RAW MPIS]: artist: {:?}, title: {:?}", raw_artist, raw_title);
                                info!("Track changed [CLEANED]: {:?}", new_track);
                                let _ = tx.blocking_send(AppEvent::TrackChanged(new_track));
                            }
                        }
                    }
                }

                if let Ok(timeline) = session.GetTimelineProperties() {
                    let raw_pos = timeline.Position().unwrap_or_default().Duration;
                    
                    let mut pos = tracker.calculate_adjusted_position(Duration::from_nanos((raw_pos as u64) * 100)).as_nanos() as i64 / 100;
                    
                    // Windows interpolates real-time via SystemTime since `Position` is a snapshot
                    if let Ok(playback_info) = session.GetPlaybackInfo() {
                        if let Ok(status) = playback_info.PlaybackStatus() {
                            if status == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing {
                                if let Ok(last_updated) = timeline.LastUpdatedTime() {
                                    if let Ok(now_since_epoch) = SystemTime::now().duration_since(UNIX_EPOCH) {
                                        // Windows FILETIME epoch: Jan 1, 1601. Unix epoch: Jan 1, 1970.
                                        // Difference in 100-nanosecond intervals: 116444736000000000
                                        let now_100ns = (now_since_epoch.as_nanos() / 100) as i64 + 116444736000000000;
                                        let updated_100ns = last_updated.UniversalTime;
                                        
                                        let diff = now_100ns - updated_100ns;
                                        if diff > 0 && diff < 86_400_000_000 { // Max 24h sanity check
                                            pos += diff;
                                        }
                                        trace!("Syncing Windows position: last_updated={} now={} pos={} (raw={} offset={:?})", 
                                              updated_100ns, now_100ns, pos, raw_pos, tracker.track_start_offset);
                                    }
                                }
                            }
                        }
                    }

                    if let Ok(end_time) = timeline.EndTime() {
                        if end_time.Duration > 0 && pos > end_time.Duration {
                            pos = end_time.Duration;
                        }
                    }

                    // Avoid sending negative or zero positions erroneously.
                    if pos > 0 {
                        let pos_duration = Duration::from_nanos((pos as u64) * 100);
                        let _ = tx.blocking_send(AppEvent::PositionUpdated(pos_duration));
                    }
                }
            }

            thread::sleep(POLL_INTERVAL);
        }
    });
}
