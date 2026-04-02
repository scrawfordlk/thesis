.PHONY: all clean test

RUSTC = rustc
EDITION = 2024
LLVM_LINK = llvm-link
LLC = llc

all: program

# Emit LLVM-IR and (type) metadata of wrapper
wrapper.ll libwrapper.rmeta: wrapper.rs
	$(RUSTC) --edition $(EDITION) --emit=llvm-ir,metadata --crate-type=rlib -o wrapper $<

# Bootstrap the core using rustc
# This requires the wrapper's metadata for type information
core.ll: core.rs libwrapper.rmeta
	$(RUSTC) --edition $(EDITION) --emit=llvm-ir --extern wrapper=libwrapper.rmeta -o $@ $<

combined.ll: core.ll wrapper.ll
	$(LLVM_LINK) -S $^ -o $@

combined.o: combined.ll
	$(LLC) -filetype=obj -relocation-model=pic $< -o $@

# Step 5: Link object with Rust stdlib
# Use a cdylib shim to avoid duplicate main - links as a library
program: combined.o
	@echo '#![no_main]' > _link.rs
	$(RUSTC) --edition $(EDITION) _link.rs \
		-C link-arg=$(CURDIR)/combined.o \
		-o $@
	@rm -f _link.rs

test: program
	@echo "=== Test ==="
	./program

clean:
	rm -f libwrapper.rmeta program *.ll *.o _link.rs

