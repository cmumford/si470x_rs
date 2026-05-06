RELEASE=--release

.PHONY: default
default: build

.PHONY: clean
clean:
	cargo clean && cd examples/esp32 && cargo clean

.PHONY: build
build:
	cargo build --features=async

.PHONY: test
test:
	cargo +stable test --features=test

.PHONY: docs
docs:
	cargo doc --no-deps --document-private-items --open --features=async

.PHONY: clippy
clippy:
	cargo clippy --features=async --

.PHONY: check
check:
	cargo check --features sync

.PHONY: machete
machete:
	cargo machete --with-metadata

.PHONY: publish
publish:
	cargo publish --features=async
