# Contributing

Thank you for your interest in contributing to `cargo-lean`.

## Prerequisites

- Rust 1.85+ — the project uses the 2024 edition.
- [Just](https://just.systems/) — recipe runner for local development commands.

## Getting Started

After cloning the repository, follow these steps to set up the project:

```bash
git clone git@github.com:raphaelstolt/cargo-lean.git && cd cargo-lean
```

Install Rust dependencies. Cargo automatically resolves and downloads all dependencies declared in `Cargo.toml`. Simply
run any Cargo command to trigger the download:

```bash
cargo check
```

This compiles just enough to verify that all dependencies are available and the project builds correctly. For a full build:

```bash
cargo build
```

Install the `cargo-lean` subcommand. To use `cargo-lean` as a Cargo subcommand:

```bash
cargo install --path .
```

Verify the setup. Run all checks to confirm everything works:

```bash
just check
```

## Code Quality

This project uses [Just](https://just.systems/) to manage common development commands.

See all available recipes with:

```bash
just --list
```

### Check (recommended)

Run formatting, linting, leanness validation, and tests in one command:

```bash
just check
```

### Formatting

Check formatting (enforced in CI):

```bash
just fmt
```

### Linting

Check for common mistakes and improve Rust idioms (warnings are errors):

```bash
just clippy
```

### Validating leanness

Validate the `package.exclude` list against the current repository state:

```bash
just lean
```

### Testing

Run the full test suite (unit + integration tests):

```bash
just test
```

### Package preview

List the files that would be included in a `cargo package` artifact:

```bash
just package
```

## Pull Requests

Please open pull requests against the `main` branch from a dedicated feature branch. Use descriptive branch names that
reflect the change, such as:

- `feat/history-command`
- `fix/update`
- `docs/update-readme`

Before opening a PR, ensure:

1. `just check` passes locally (fmt, clippy, lean, and tests).
2. `CHANGELOG.md` is updated under the `[Unreleased]` section with a summary of changes.
3. All commit messages follow [Conventional Commits](https://www.conventionalcommits.org/).

## Commit Messages

Please follow [Conventional Commits](https://www.conventionalcommits.org/) for all commit messages. This keeps the history readable
and enables automated changelog generation. You can inspect the message schema for this project via `git log --oneline -n 10`.

Common types used in this project:

- `feat:` — new features or behavior changes
- `fix:` — bug fixes
- `docs:` — documentation updates
- `refactor:` — code changes that neither fix bugs nor add features
- `test:` — adding or updating tests
- `chore:` — maintenance tasks (dependencies, config, etc.)
