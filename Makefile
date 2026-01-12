RELEASE=--release

.PHONY: default
default: examples

.PHONY: simple-esp32c3
simple-esp32c3:
	cargo build --features=esp32c3 --target riscv32imc-unknown-none-elf $(RELEASE) --example simple

.PHONY: simple-esp32c6
simple-esp32c6:
	cargo build --features=esp32c6 --target riscv32imac-unknown-none-elf $(RELEASE) --example simple

.PHONY: simple-esp32h2
simple-esp32h2:
	cargo build --features=esp32h2 --target riscv32imac-unknown-none-elf $(RELEASE) --example simple

.PHONY: simple-esp32
simple-esp32:
	cargo build --features=esp32 --target xtensa-esp32-none-elf $(RELEASE) --example simple

.PHONY: simple-esp32s2
simple-esp32s2:
	cargo build --features=esp32s2 --target xtensa-esp32s2-none-elf $(RELEASE) --example simple

.PHONY: simple-esp32s3
simple-esp32s3:
	cargo build --features=esp32s3 --target xtensa-esp32s3-none-elf $(RELEASE) --example simple

.PHONY: simple-esp32c3-async
simple-esp32c3-async:
	cargo build --features=esp32c3,async --target riscv32imc-unknown-none-elf $(RELEASE) --example simple_async

.PHONY: simple-esp32c6-async
simple-esp32c6-async:
	cargo build --features=esp32c6,async --target riscv32imac-unknown-none-elf $(RELEASE) --example simple_async
.PHONY: simple-esp32h2-async
simple-esp32h2:
	cargo build --features=esp32h2,async --target riscv32imac-unknown-none-elf $(RELEASE) --example simple

.PHONY: simple-esp32-async
simple-esp32:
	cargo build --features=esp32,async --target xtensa-esp32-none-elf $(RELEASE) --example simple

.PHONY: simple-esp32s2-async
simple-esp32s2:
	cargo build --features=esp32s2,async --target xtensa-esp32s2-none-elf $(RELEASE) --example simple

.PHONY: simple-esp32s3-async
simple-esp32s3:
	cargo build --features=esp32s3,async --target xtensa-esp32s3-none-elf $(RELEASE) --example simple

.PHONY: examples
examples: simple-esp32c3 simple-esp32c6 simple-esp32h2 simple-esp32 simple-esp32s2 simple-esp32s3 \
		  simple-esp32c3-async simple-esp32c6-async simple-esp32h2-async simple-esp32-async simple-esp32s2-async simple-esp32s3-async

.PHONY: flash-simple-esp32c6
flash-simple-esp32c6:
	cargo espflash flash --example simple --monitor --baud=921600 --target riscv32imac-unknown-none-elf $(RELEASE) --features=esp32c6

.PHONY: flash-simple-esp32c6-async
flash-simple-esp32c6-async:
	cargo espflash flash --example simple_async --monitor --baud=921600 --target riscv32imac-unknown-none-elf $(RELEASE) --features=esp32c6,async

.PHONY: clean
clean:
	cargo clean
