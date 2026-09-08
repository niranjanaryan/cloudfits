.PHONY: build test run cli tui system providers compare clean format lint install

build:
	cargo build --release

test:
	cargo test

run:
	cargo run -p cloudfits-tui

cli:
	cargo run -p cloudfits-cli --

# Example invocations
system:
	cargo run -p cloudfits-cli -- system

providers:
	cargo run -p cloudfits-cli -- providers

compare:
	cargo run -p cloudfits-cli -- compare --model llama-3.1-8b

clean:
	cargo clean

format:
	cargo fmt

lint:
	cargo clippy -- -D warnings

install:
	cargo install --path cloudfits-cli --locked
	cargo install --path cloudfits-tui --locked
