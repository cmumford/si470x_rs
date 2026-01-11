CHIP=esp32c6
FEATURES=--features=$(CHIP)

.PHONY: examples
examples:
	cargo build --examples $(FEATURES)

.PHONY: clean
clean:
	cargo clean