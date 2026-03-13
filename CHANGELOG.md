# Changelog

All notable changes to this project will be documented in this file.

## [0.3.0](https://github.com/guptarohit/mfp/compare/v0.2.0...v0.3.0) (2026-03-13)

### Added

- Interactive playback controls during streaming playback:
  - pause/resume with `Space`
  - stop with `q`
  - stop with `Ctrl+C`
  - seek forward 10 seconds with `Right` or `l`
  - seek backward 10 seconds with `Left` or `h`
- Interactive volume control with `+` / `=` and `-` / `_`
- Inline playback status feedback such as `[PAUSED]` and volume level updates
- Automatic release workflow with `release-please`
- Semantic PR title validation workflow

### Changed

- Split CI and release automation into dedicated GitHub Actions workflows
- Added macOS ARM release builds
- Improved terminal interaction with raw mode restoration and non-TTY fallback
- Updated README with playback controls documentation

### Fixed

- Replaced `minimp3` with `minimp3_fixed` to avoid the undefined behavior crash caused by `slice-deque` on newer Rust toolchains
- Added HTTP timeout handling and graceful recovery for seek operations
- Preserved playback continuity when seek requests fail
