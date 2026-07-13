all: clippy

.PHONY: clippy
clippy:
	@cargo clippy --tests --bins --workspace -- --no-deps

.PHONY: build
build:
	@cargo build --workspace

.PHONY: test
test:
	@cargo nextest run --workspace
