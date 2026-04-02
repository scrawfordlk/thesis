.PHONY: all llvm riscv test-llvm test-riscv clean

RUSTC = rustc
EDITION = 2024
LLVM_LINK = llvm-link
LLC = llc

# Feature flag for standalone compilation
STANDALONE = --cfg 'feature="standalone"'

# RISC-V settings
RISCV_TARGET = riscv64gc-unknown-linux-musl
RISCV_FLAGS = -C target-feature=+crt-static -C linker=rust-lld -C link-self-contained=yes

all: llvm

# === LLVM-IR backend (x86-64) ===

llvm: program-llvm

wrapper.ll libwrapper.rmeta: wrapper.rs
	$(RUSTC) --edition $(EDITION) --emit=llvm-ir,metadata --crate-type=rlib -o wrapper $<

core.ll: core.rs libwrapper.rmeta
	$(RUSTC) --edition $(EDITION) $(STANDALONE) --emit=llvm-ir \
		-L . --extern wrapper=libwrapper.rmeta -o $@ $<

combined.ll: core.ll wrapper.ll
	$(LLVM_LINK) -S $^ -o $@

combined.o: combined.ll
	$(LLC) -filetype=obj -relocation-model=pic $< -o $@

program-llvm: combined.o
	@echo '#![no_main]' > _link.rs
	$(RUSTC) --edition $(EDITION) _link.rs -C link-arg=$(CURDIR)/combined.o -o $@
	@rm -f _link.rs

test-llvm: program-llvm
	@echo "=== LLVM-IR test ==="
	./program-llvm

# === RISC-V backend ===

riscv: program-riscv

libwrapper-riscv.rlib: wrapper.rs
	$(RUSTC) --edition $(EDITION) --target=$(RISCV_TARGET) --crate-type=rlib \
		$(RISCV_FLAGS) -o $@ $<

program-riscv: core.rs libwrapper-riscv.rlib
	$(RUSTC) --edition $(EDITION) $(STANDALONE) --target=$(RISCV_TARGET) \
		$(RISCV_FLAGS) -L . --extern wrapper=libwrapper-riscv.rlib -o $@ $<

test-riscv: program-riscv
	@echo "=== RISC-V test ==="
	qemu-riscv64 ./program-riscv

# === Utilities ===

test: test-llvm

clean:
	rm -f *.rlib *.rmeta *.ll *.o program-llvm program-riscv _link.rs
