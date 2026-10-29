fmt:
	cargo fmt -- --check

clippy:
	cargo clippy -- -D warnings

test:
	cargo test

check: fmt clippy lean test

build:
	cargo build

build-release:
	cargo build --release

run:
	cargo run

lean:
    cargo install --path . && cargo-lean lint

package:
	cargo package --allow-dirty --list
