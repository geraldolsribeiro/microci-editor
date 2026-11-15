.PHONY: bacon
bacon:
	cp microCI.yml.sample .microCI.yml
	bacon run

.PHONY: build
build:
	cargo build

.PHONY: build_wasm
build_wasm:
	cargo build --target=wasm32-unknown-unknown

.PHONY: publish
publish:
	cargo fmt --check
	cargo test
	cargo publish

.PHONY: install_bacon
install_bacon:
	cargo install --locked bacon
	bacon --init
