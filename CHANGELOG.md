# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed
- Global `.gitignore` support: Artefact detection now accounts for patterns in the global gitignore excludes file
  (configured via core.excludesFile or the XDG default at ~/.config/git/ignore), in addition to the local .gitignore.

## [0.1.0] - 2026-10-27

### Added

- **`lint` command**: Checks whether all unnecessary repository artefacts are listed in `Cargo.toml` `[package].exclude`.
- **`create` command**: Adds missing artefacts to the `Cargo.toml` exclude list, preserving existing entries.
- **`update` command**: Replaces the `Cargo.toml` exclude list with only the artefacts that currently exist in the repository.
- **`--format json` option**: All commands support JSON output for scripting and interoperability.
- **`--diff` flag** (lint command): Shows a diff-style preview of which artefacts would be added to the exclude list.
- **`--dry-run` flag** (create/update commands): Preview changes without modifying the `Cargo.toml` file.
- **`--patterns` / `-p` option**: Override the default list of artefact patterns with custom ones.
- **`--manifest-path` / `-m` option**: Specify a custom path to the `Cargo.toml` file.