.PHONY: all
all:
	bacon run

.PHONY: install_bacon
all:
install_bacon:
	cargo install --locked bacon
	bacon --init
