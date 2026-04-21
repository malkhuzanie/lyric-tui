# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [v0.1.1] - 2026-04-21

This patch introduces a shared architectural component to mitigate severe cross-platform temporal drift and resolves native input duplication on Windows.

### Added
- Introduce the `TimelineTracker` struct to centralise playback offset logic.

### Fixed
- Resolve cross-platform timestamp drift caused by Chromium/WebKit autoplay anomalies.
- Filter `crossterm` key release events on Windows to eliminate modal flickering.
- Implement GSMTC position interpolation to rectify static timeline snapshots on Windows.

## [v0.1.0] - 2026-04-17

### Added
- Initial public release of `lyric-tui`.
- Establish core async orchestration using `tokio` and `ratatui`.
- Implement `LrclibProvider` for fetching time-synced `.lrc` metadata.
- Implement cross-platform media monitoring for Linux (`mpris`) and Windows (GSMTC).