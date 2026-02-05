.PHONY: build test lint fmt clean check all

all: fmt lint test build

build:
	cargo build --release

test:
	cargo test

lint:
	cargo clippy

fmt:
	cargo fmt

fmt-check:
	cargo fmt --check

check:
	cargo check

clean:
	cargo clean
