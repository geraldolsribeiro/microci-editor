.PHONY: all
all:
	cp microCI.yml.sample .microCI.yml
	bacon run

.PHONY: install_bacon
all:
install_bacon:
	cargo install --locked bacon
	bacon --init
