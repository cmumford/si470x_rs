# ESP_CHIPS := esp32 esp32c3 esp32c6 esp32h2 esp32s2 esp32s3
ESP_CHIPS := esp32c3 esp32c6 esp32h2

.PHONY: default
default: simple

.PHONY: simple-esp32c3
simple-esp32c3:
	cargo build --features=esp32c3 --target riscv32imc-unknown-none-elf --release --example simple

.PHONY: simple-esp32c6
simple-esp32c6:
	cargo build --features=esp32c6 --target riscv32imac-unknown-none-elf --release --example simple

.PHONY: simple-esp32h2
simple-esp32h2:
	cargo build --features=esp32h2 --target riscv32imac-unknown-none-elf --release --example simple

.PHONY: simple
simple: simple-esp32c3 simple-esp32c6 simple-esp32h2

.PHONY: clean
clean:
	cargo clean
