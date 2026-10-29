# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2026-10-30

### Changed
- `--patterns` / `-p` option now accepts a single space-separated string (e.g., `-p "tests/ docs/ PROMPTS.md"`) instead
of multiple repeated flags.

## [0.2.0] - 2026-10-29

### Fixed
- Global `.gitignore` support: Artefact detection now accounts for patterns in the global gitignore excludes file
  (configured via core.excludesFile or the XDG default at ~/.config/git/ignore), in addition to the local .gitignore.
- Tilde expansion: Paths containing `~` in `core.excludesFile` (e.g. `~/.config/git/.gitignore`) are now
  correctly resolved to the user's home directory.
- Case-insensitive matching: Exclude patterns are now compared case-insensitively, so `changelog.md` matches
  `CHANGELOG.md` in the exclude list, handling differences between filesystem casing and pattern names.
- `update` command preserves custom exclude entries: Running `update` now keeps entries already in
  `[package].exclude` that still exist in the repository, even if they are not in the built-in default patterns.
  Stale entries (files no longer present) are still removed as expected.

## [0.1.0] - 2026-10-27

### Added

- __`lint` command__: Checks whether all unnecessary repository artefacts are listed in `Cargo.toml` `[package].exclude`.
- __`create` command__: Adds missing artefacts to the `Cargo.toml` exclude list, preserving existing entries.
- __`update` command__: Replaces the `Cargo.toml` exclude list with only the artefacts that currently exist in the repository.
- __`--format json` option__: All commands support JSON output for scripting and interoperability.
- __`--diff` flag__ (lint command): Shows a diff-style preview of which artefacts would be added to the exclude list.
- __`--dry-run` flag__ (create/update commands): Preview changes without modifying the `Cargo.toml` file.
- __`--patterns` / `-p` option__: Override the default list of artefact patterns with custom ones.
- __`--manifest-path` / `-m` option__: Specify a custom path to the `Cargo.toml` file.
