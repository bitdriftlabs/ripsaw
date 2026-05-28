all: clippy

.PHONY: clippy
clippy:
	cargo clippy --tests --bins --workspace

.PHONY:
build:
	cargo build --workspace

.PHONY:
test:
	cargo nextest run --workspace
