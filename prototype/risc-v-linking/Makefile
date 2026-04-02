.PHONY: all clean test

RUSTC = rustc
EDITION = 2024
TARGET = riscv64gc-unknown-linux-musl
RUSTFLAGS = -C target-feature=+crt-static -C linker=rust-lld -C link-self-contained=yes

all: program

# Step 1: Compile wrapper to rlib
libwrapper.rlib: wrapper.rs
	$(RUSTC) --edition $(EDITION) --target=$(TARGET) --crate-type=rlib \
		$(RUSTFLAGS) -o $@ $<

# Step 2: Compile core, linking with wrapper
program: core.rs libwrapper.rlib
	$(RUSTC) --edition $(EDITION) --target=$(TARGET) \
		$(RUSTFLAGS) --extern wrapper=libwrapper.rlib -o $@ $<

test: program
	@echo "=== Test ==="
	qemu-riscv64 ./program

clean:
	rm -f libwrapper.rlib program
