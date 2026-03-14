# Changelog

All notable changes to this project will be documented in this file.

## [0.3.2](https://github.com/guptarohit/mfp/compare/v0.3.1...v0.3.2) (2026-03-14)


### Fixed

* avoid file#label syntax in gh release upload for Linux compatibility ([01c69ee](https://github.com/guptarohit/mfp/commit/01c69ee252a282ffe8d8475bd6aaad8d6216cfc3))
* songs cut off early when RSS duration is shorter than actual audio ([2f2e88b](https://github.com/guptarohit/mfp/commit/2f2e88b8394be93394d6beafa4e2102cfa653a46)), closes [#1](https://github.com/guptarohit/mfp/issues/1)
* songs cut off early when RSS duration is shorter than actual audio ([#21](https://github.com/guptarohit/mfp/issues/21)) ([f73f532](https://github.com/guptarohit/mfp/commit/f73f532a290f24f5638eb8079d47bc8dbfa7990a))

## [0.3.1](https://github.com/guptarohit/mfp/compare/v0.3.0...v0.3.1) (2026-03-13)


### Fixed

* escape # in asset filename for rebuild workflow ([6cc23ff](https://github.com/guptarohit/mfp/commit/6cc23ff4b937f6a7eb25eb5b9c9cd1645ef4f4e7))
* escape # in asset filename for rebuild workflow ([#18](https://github.com/guptarohit/mfp/issues/18)) ([cc71a27](https://github.com/guptarohit/mfp/commit/cc71a279381ff4e987cd275ab79e6d3772ce0252))
* install libasound2-dev in publish-crate, add recovery workflow ([15faaf9](https://github.com/guptarohit/mfp/commit/15faaf9cc8fca6ae24aa91756cd8879302d34afc))
* install libasound2-dev in publish-crate, add recovery workflow ([#17](https://github.com/guptarohit/mfp/issues/17)) ([8676ab4](https://github.com/guptarohit/mfp/commit/8676ab4465813a25eafa039da258225d8a4de8b4))
* stop prefixing release tags with component name ([3f30384](https://github.com/guptarohit/mfp/commit/3f30384434060558ac1a113cde3c63c17775da41))

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
