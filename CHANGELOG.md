# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project aims
to follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html) once it
reaches 1.0. Until then, minor releases may include changes that would be
breaking under strict semver; those are called out explicitly.

## [Unreleased]

### Changed

- `mog market list` and `mog market show` are now backed by a freshness-checked
  scan index cached at `<library>/.cache/scan-index.json`. A repeat listing reuses
  the built index instead of re-reading and re-parsing every mog, so on a
  ~230-mog library `list` drops from roughly 185ms to about 70ms and `show`
  falls to the process-startup floor (about 30ms). The cache rebuilds
  automatically whenever a mog is added, removed, or edited (its fingerprint is
  the mog count plus the newest mog mtime); a stale or corrupt index is
  ignored and recomputed, and a read-only library simply runs uncached.

## [0.1.0] - 2026-08-26

First public release.

### Added

- The `.mog` script format: an ordered `steps` pipeline of find/replace and
  text-transform actions, run over one or many files.
- A broad action catalog: literal and regex replace (with `$1` backrefs and
  `$#` / `$lineno` / `$uuid` per-occurrence tokens), line operations (filter,
  sort including `sort_ip` / `sort_versions` / `sort_imports`, dedupe, number,
  join), blank/whitespace, EOL conversion, case conversion (including
  `to_camel` / `to_pascal` identifier recasing), encode and escape, and
  reading-oriented format conversion across JSON/CSV/YAML/TOML/XML/logfmt/INI and
  more.
- Scoping: apply a step to a block, a line range, a delimited field, or a
  character range; plus line-match filters.
- Two-input compares (`diff`, `reconcile`, `intersect`, `subtract`) against a
  named second source.
- Detectors and redaction (`detect_secrets`, `detect_pii`), an `assert` gate,
  mail-merge from external `sources`, and script composition with `run_mog`.
- A curated, tested mog library, discoverable and runnable through the CLI.
- Per-mog `summary` and `task_phrases` metadata for human browsing and search.
- Cross-platform CLI: in-place / out-dir / stdout writing, `--dry-run` and
  `--diff` preview, backups, parallel processing, and JSON output for tooling.
- Prebuilt release binaries for Linux, macOS, and Windows (x64 and arm64) via CI.

[Unreleased]: https://github.com/kjustinkeener/mog/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/kjustinkeener/mog/releases/tag/v0.1.0
