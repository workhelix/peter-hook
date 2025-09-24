# Changelog

All notable changes to this project will be documented in this file.

## [1.0.8] - 2025-09-23

### Fixed
- Fixed install script bug where log messages were outputting to stdout instead of stderr, causing version detection to fail with "bad range in URL" error

## [0.3.0] - 2025-09-10

### Added
- Expose changed files to hook commands via environment variables:
  - `CHANGED_FILES`: space-delimited list of repo-relative paths
  - `CHANGED_FILES_LIST`: newline-delimited list of repo-relative paths
  - `CHANGED_FILES_FILE`: absolute path to a temporary file containing the newline-delimited list
- Per-hook filtering of changed files based on the hook's `files = [..]` patterns
- Documentation in README for the new environment variables

### Notes
- Variables are populated when running with `--files`; otherwise they are set but empty (`CHANGED_FILES_FILE` is an empty string).
- Backward compatible; no breaking changes.
