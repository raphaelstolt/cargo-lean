# cargo-lean

Validate and manage `Cargo.toml` package exclude lists to keep your published crates lean.

## Why?

When you publish a Rust crate to crates.io, Cargo packages up every file in your repository that Git tracks — unless
you explicitly exclude it. That means your published crate can unintentionally include development artefacts like
IDE settings, test suites, build output, documentation drafts, and CI configuration. These files bloat the crate
download size, slow down `cargo add` and `cargo install`, and expose internal project structure to downstream users.

`cargo-lean` makes sure your `[package].exclude` list is complete so only the source code your users need ends up
in the published crate.

## Installation

### From crates.io

```bash
cargo install cargo-lean
```

### From source

```bash
git clone git@github.com:raphaelstolt/cargo-lean.git
cd cargo-lean
cargo install --path .
```

This places the `cargo-lean` binary in `~/.cargo/bin`, which should be on your PATH.

## Usage

```
cargo-lean <COMMAND>
```

### Commands

| Command  | Description                                                        |
|----------|--------------------------------------------------------------------|
| `lint`   | Check if all repository artefacts are excluded.                    |
| `create` | Add missing artefacts to the exclude list.                         |
| `update` | Replace the exclude list, preserving custom entries that still exist. |

### `lint` — Check exclude list completeness

Verifies that all detected repository artefacts are listed in `Cargo.toml` excludes.

```bash
# Check the current directory
cargo-lean lint

# Check a specific manifest
cargo-lean lint --manifest-path /path/to/Cargo.toml

# Show a diff of what would be added
cargo-lean lint --diff

# JSON output for scripting
cargo-lean lint --format json
```

__`--diff` flag:__ Shows which artefacts would be added to the exclude list, using `+` prefixes (similar to a unified
diff). This is useful for previewing changes before running `create`.

### `create` — Add missing exclusions

Adds any detected artefacts that are not yet in the exclude list. Preserves existing entries.

```bash
# Dry run: see what would be added
cargo-lean create --dry-run

# Apply changes
cargo-lean create

# JSON output
cargo-lean create --format json
```

### `update` — Replace exclude list

Replaces the entire exclude list with the artefacts that currently exist in the repository, merging built-in default
patterns with any custom entries already in `[package].exclude`. This removes stale entries (files no longer present)
and adds new ones, while preserving custom exclude entries that still exist.

```bash
# Dry run: see what would change
cargo-lean update --dry-run

# Apply changes
cargo-lean update

# JSON output
cargo-lean update --format json
```

## Options and flags 

All commands support these common options and flags:

| Option / Flag       | Description                                      |
|---------------------|--------------------------------------------------|
| `-m, --manifest-path <PATH>` | Path to `Cargo.toml` (default: `Cargo.toml`) |
| `--format <FORMAT>` | Output format: `text` or `json` (default: `text`) |
| `-p, --patterns <PATTERN>`   | Custom artefact patterns (can be repeated)     |
| `--dry-run`         | Preview changes without modifying files (`create`, `update`) |
| `--diff`            | Show diff-style output (`lint`)                  |

Note: When using `--manifest-path`, the working directory is derived from the parent directory of the provided
`Cargo.toml`. For example, `--manifest-path /path/to/Cargo.toml` will check for artefacts, `.gitignore`, and other
files relative to `/path/to/`.

## Default artefact patterns

By default, `cargo-lean` checks for these 30 patterns (directories end with `/`):

```
.gitignore, .gitattributes, .github/, .idea/, .vscode/, .cargo/, .config/,
tests/, test/, benches/, examples/, docs/, doc/, target/,
CHANGELOG.md, changelog.md, CONTRIBUTING.md, contributing.md,
Justfile, justfile, Makefile, makefile,
.editorconfig, .pre-commit-config.yaml, .markdownlint.yaml, .yamllint,
.dockerignore, Dockerfile, docker-compose.yml, docker-compose.yaml
```

Note: `.git` is not included as it cannot be tracked by Git.

Override them with `--patterns`:

```bash
cargo-lean lint -p .idea/ -p target/
```

## `.gitignore` awareness

All commands automatically read the `.gitignore` file in the manifest directory. Artefacts already covered by
`.gitignore` entries are excluded from the results, since Git will not track them anyway.

- __`lint`__: Will not flag artefacts that are already in `.gitignore`.
- __`create`__: Will not add artefacts that are already in `.gitignore` to the exclude list.
- __`update`__: Will not include artefacts that are already in `.gitignore` in the new exclude list.

When using `--format json`, the `gitignored` array lists which patterns were covered by `.gitignore`.

## CI integration

Add `cargo-lean lint` to your CI pipeline to enforce that exclude lists stay complete:

```yaml
# Example GitHub Actions step
- name: Check Cargo.toml package excludes
  run: cargo-lean lint
```

## JSON output

All commands support `--format json` for scripting and automation. Example `lint` output:

```json
{
  "valid": false,
  "missing": [".idea"],
  "excluded": ["tests/", "CHANGELOG.md"],
  "gitignored": ["target/"],
  "total_patterns": 30
}
```

## Development

See [CONTRIBUTING.md](CONTRIBUTING.md) for development instructions.

## License

This project is licensed under the MIT license. Please see [LICENSE.md](LICENSE.md) for more details.

## Related projects

- [lean-package-validator](https://github.com/raphaelstolt/lean-package-validator) — The `PHP` equivalent.
- [py-lean](https://github.com/raphaelstolt/py-lean) — The `Python` equivalent.
