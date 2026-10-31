# Run Rust formatter in check mode
fmt:
	cargo fmt -- --check

# Run Clippy linter with warnings as errors
clippy:
	cargo clippy -- -D warnings

# Run the test suite
test:
	cargo test

# Install locally and run cargo-lean lint against itself
lean:
	cargo install --path . && cargo-lean lint

# Preview package contents without cleaning
package:
	cargo package --allow-dirty --list

# Run all checks: formatting, linting, cargo-lean lint, and tests
check: fmt clippy lean test

# Preview a release (dry run)
release-dry-run level="patch":
	cargo release {{level}}

# Execute a release
release level="patch":
	cargo release {{level}} --execute
