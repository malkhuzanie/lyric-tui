### Architecture Overview: The `TimelineTracker`

The `TimelineTracker` is a cross-platform architectural component designed to resolve severe temporal drift during media playback. It sits as a middleware layer between the raw data provided by the Operating System's media APIs (Linux MPRIS or Windows GSMTC) and the application's internal state.

### The Underlying Problem: Why is this necessary?

When a user listens to a native application like Spotify, the application correctly resets its internal playback clock to `0:00` at the start of every new song.

However, when a user listens to an autoplaying playlist on YouTube via a web browser (Chromium, WebKit, or Firefox), a unique anomaly occurs:

1. To conserve memory, the browser does not load a new webpage for the next video. It simply pipes the new audio stream into the existing HTML5 `<video>` element.

2. The browser updates the Title and Artist metadata, but it fails to reset the underlying media clock.

3. Consequently, the OS media APIs report the _cumulative_ time of the entire playlist rather than the time of the current track.


If a user listens to a three-minute song, the next song will erroneously begin its timeline at `3:01`. Without intervention, `lyric-tui` would attempt to display lyrics for the third minute of the song the moment it begins playing.

### The Concept: How it functions

The `TimelineTracker` functions by employing a **Stale Offset Mitigation Strategy**. It does not attempt to alter the OS media session; rather, it intercepts the corrupted data, calculates the margin of error, and mathematically corrects the timestamp before passing it to the user interface.

### Step-by-Step Logical Flow

1. **Instantiation:** When the media monitor thread begins, a single `TimelineTracker` is instantiated. If the user switches media players (e.g., pausing Spotify and opening Chrome), the tracker is purged via `reset_player()`.

2. **Track Change Detection:** At a regular interval (~200ms), the system polls the OS for metadata. This metadata is passed to `process_track_change()`.

3. **Offset Capture:** If `process_track_change()` determines that the track has genuinely changed (the Title or Artist differs from the previous poll), it inspects the raw timeline position. If the position is greater than zero, the tracker assumes a browser anomaly has occurred and saves this exact timestamp as the `track_start_offset`.

4. **Position Adjustment:** When the system subsequently polls the OS for the current playback position, it passes the raw metric to `calculate_adjusted_position()`. This function simply subtracts the `track_start_offset` from the raw position.

5. **Self-Correction:** If the user manually clicks the timeline slider on YouTube (seeking back to the beginning), the raw position will suddenly drop below the `track_start_offset`. The tracker detects this, assumes the browser has rectified its state, and resets the offset back to `0`.


### Integration with OS Implementations

The beauty of the `TimelineTracker` is its agnostic design; it integrates flawlessly into both `linux.rs` and `windows.rs` using the exact same API.

**Example Integration (Linux MPRIS):**

- **Initialisation:** The tracker is created before the polling loop begins (`let mut tracker = super::common::TimelineTracker::new();`).

- **Player Switch:** If the `find_player` logic detects a new D-Bus identity, we immediately invoke `tracker.reset_player()` to ensure stale offsets do not bleed into the new application.

- **Metadata Polling:** We retrieve the metadata, clean it, and extract the raw position. We feed this into `tracker.process_track_change(new_track, current_raw)`. If this returns `true`, we broadcast the `TrackChanged` event to the application.

- **Position Polling:** Crucially, position polling now occurs _after_ metadata polling. We retrieve the raw D-Bus position (`raw_pos`) and immediately pass it through our middleware: `let pos = tracker.calculate_adjusted_position(raw_pos);`. The application is entirely shielded from the browser's faulty metrics.