fmt:
	cargo fmt -- --check

clippy:
	cargo clippy -- -D warnings

test:
	cargo test

lean:
	cargo install --path . && cargo-lean lint

package:
	cargo package --allow-dirty --list

check: fmt clippy lean test

# Preview a release (dry run)
release-dry-run level="patch":
	cargo release {{level}}

# Execute a release
release level="patch":
	cargo release {{level}} --execute
