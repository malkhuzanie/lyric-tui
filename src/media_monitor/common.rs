// src/media_monitor/common.rs
use regex::Regex;
use std::sync::LazyLock;
use crate::app::TrackInfo;
use std::time::Duration;

static TRACK_PREFIX_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*(?:\d+|[A-D]\d+)[.\-]\s*").expect("Hard-coded regex is valid")
});

/// Sanitises a raw track title by stripping superfluous numerical prefixes.
pub fn clean_title(raw: &str) -> String {
    TRACK_PREFIX_RE.replace(raw, "").into_owned()
}


/// A cross-platform utility designed to mitigate temporal drift during media playback.
///
/// This tracker specifically addresses anomalies introduced by browser engines 
/// (such as Chromium or WebKit) reusing HTML5 `<video>` elements during playlist 
/// autoplay. By tracking true track transitions and capturing stale timeline 
/// metrics, this struct ensures the application calculates an accurate, zero-indexed 
/// position for the current track.
#[derive(Default)]
pub struct TimelineTracker {
    /// The currently active media track being monitored.
    pub current_track: Option<TrackInfo>,
    /// The temporal offset applied to rectify stale timeline metrics.
    pub track_start_offset: Duration,
    /// The most recently observed raw timeline position from the OS media session.
    pub last_raw_pos: Duration,
}

impl TimelineTracker {
    /// Instantiates a new, empty `TimelineTracker`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Evaluates incoming metadata to detect genuine track transitions.
    ///
    /// If a transition is detected and the underlying media session provides a stale 
    /// (non-zero) starting position, this method captures that position as an offset 
    /// to mitigate temporal drift.
    ///
    /// Returns `true` if the incoming track differs from the active track and requires 
    /// broadcasting to the broader application state.
    pub fn process_track_change(&mut self, new_track: TrackInfo, current_raw: Duration) -> bool {
        let is_true_track_transition = match self.current_track.as_ref() {
            Some(ct) => ct.artist != new_track.artist || ct.title != new_track.title,
            None => false,
        };

        if Some(&new_track) != self.current_track.as_ref() {
            if is_true_track_transition {
                if current_raw > Duration::ZERO && current_raw >= self.last_raw_pos {
                    self.track_start_offset = current_raw;
                    log::debug!("Browser stale timeline detected: offsetting by {:?}", self.track_start_offset);
                } else {
                    self.track_start_offset = Duration::ZERO;
                }
            } else {
                self.track_start_offset = Duration::ZERO;
            }
            self.current_track = Some(new_track);
            true
        } else {
            false
        }
    }

    /// Purges the current tracking state, resetting the active track and all temporal offsets.
    pub fn reset_player(&mut self) {
        self.current_track = None;
        self.track_start_offset = Duration::ZERO;
        self.last_raw_pos = Duration::ZERO;
    }

    /// Computes the true playback position by subtracting the captured offset from the raw metric.
    ///
    /// This method also ensures the resultant position does not erroneously exceed the 
    /// known duration of the active track, and clears the offset if the underlying 
    /// media session rectifies its own timeline.
    pub fn calculate_adjusted_position(&mut self, raw_pos: Duration) -> Duration {
        self.last_raw_pos = raw_pos;
        if raw_pos < self.track_start_offset {
            self.track_start_offset = Duration::ZERO;
        }
        
        let mut pos = raw_pos.saturating_sub(self.track_start_offset);
        
        if let Some(track) = &self.current_track {
            if let Some(len) = track.length {
                if pos > len {
                    pos = len;
                }
            }
        }
        pos
    }
}
