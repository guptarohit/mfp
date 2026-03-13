# Contributing

Thanks for contributing to `mfp`.

## Before opening a PR

- run `cargo fmt`
- run `cargo clippy`
- run `cargo test --release`
- keep changes focused and backward compatible where possible

On Linux, CI also requires `libasound2-dev`.

## Pull request titles

This repository uses Conventional Commit style PR titles because release
automation relies on them.

Examples:

- `feat: add interactive volume control`
- `fix: handle Ctrl+C during playback`
- `docs: update README playback controls`
- `refactor: simplify playback timing logic`
- `test: add duration parsing coverage`

Recommended types:

- `feat`
- `fix`
- `docs`
- `test`
- `refactor`
- `perf`
- `build`
- `ci`
- `chore`
- `deps`
- `revert`

If a PR title is not in the expected format, maintainers may edit it before
merge.

## Commit messages

Conventional Commit style commit messages are welcome, but they are not
required.

Release automation relies primarily on the pull request title, so contributors
do not need to rewrite individual commits just to match the release format.

Versioning notes:

- `feat` -> minor release
- `fix` -> patch release
- `!` or `BREAKING CHANGE` -> major release

## Merge guidance for maintainers

If merge commits are used, keep the final merge commit title aligned with the PR
title so release automation can infer the release correctly from git history.

## Release process

Releases are automated:

- `release-please` opens a release PR and updates `CHANGELOG.md`
- merging the release PR creates the tag and GitHub release
- GitHub Actions builds release binaries and uploads them to the release
- GitHub Actions publishes the crate to crates.io

Please do not manually edit `CHANGELOG.md` for normal releases unless a
maintainer is intentionally correcting release notes.
