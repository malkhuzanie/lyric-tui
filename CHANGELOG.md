# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [v0.2.0] - 2026-05-12

### Added
- **Crates.io Support**: Automated publishing of the application to crates.io via GitLab CI.
- **Full-Screen Mode**: Toggle a lyrics-only view using the `f` keybind.
- **Dual-Caching Strategy**: Manual searches cache lyrics under both the search query and the original player metadata to eliminate redundant API requests.

### Changed
- Replaced the standard progress bar with a custom segmented paragraph (`▊`) for uniform rendering across varying terminal environments.
- Enhanced metadata sanitisation to proactively strip lifespan/year annotations prior to network requests.
- Track progress, album info, and elapsed time formatting are now preserved during manual searches.
- Manual searches now reuse existing cached lyrics by default.

### Fixed
- Re-enabled auto-scroll automatically following a successful manual search.
- Refactored Windows SMTC tracking with dynamic offset interpolation to fix desynchronisation when transitioning between tracks on Chromium browsers.

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