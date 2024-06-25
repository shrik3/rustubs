# QUIETER SUBMAKE
VERBOSE=@
MAKEFLAGS += --no-print-directory

BUILD = build
ARCH = x86_64
NASMFLAGS = -w-zeroing -f elf64
LINKER_SCRIPT = ./defs/$(ARCH)-hm-linker.ld
CARGO_XBUILD_TARGET = ./defs/$(ARCH)-rustubs.json
CARGO_XBUILD_FLAGS =
RUSTC_FLAGS := -C code-model=large
LDFLAGS = -no-warn-rwx-segment -static
# ASSEMBLY SOURCE AND OBJECTS
ASM_SOURCES = $(shell find ./src -name "*.s")
ASM_OBJECTS = $(patsubst %.s,_%.o, $(notdir $(ASM_SOURCES)))
ASMOBJ_PREFIXED = $(addprefix $(BUILD)/,$(ASM_OBJECTS))
# RUST SATICLIB
RUST_OBJECT = target/$(ARCH)-rustubs/$(RUST_BUILD)/librustubs.a
# SETTING DIRS TO LOOK FOR MISSING SOURCE FILES
VPATH = $(sort $(dir $(ASM_SOURCES)))
ifneq ($(filter --release,$(CARGO_XBUILD_FLAGS)),)
    RUST_BUILD = release
else
	RUST_BUILD = debug
endif
# THE FS OBJECT TO STATICALLY LINK INTO THE KERNEL
FSIMAGE = fsimage.o

all: bootdisk.iso

bootdisk.iso : $(BUILD)/kernel
	@echo "---BUILDING BOOTDISK IMAGE ---"
	$(VERBOSE) cp $< isofiles/boot/
	$(VERBOSE) grub-mkrescue -d /usr/lib/grub/i386-pc \
		--locales=en@piglatin --themes=none \
		-o bootdisk.iso isofiles > /dev/null 2>&1

$(FSIMAGE): fs.ustar
	@echo "---BUILDING RAMFS-------------"
	$(VERBOSE) objcopy --input-target binary --output-target pe-x86-64 --binary-architecture i386 --rename-section .data=.fs $< $@

fs.ustar: progs
	@echo "---CREATING USTAR ARCHIVE ----"
	$(VERBOSE) @tar -cf $@ --format=ustar --totals docs/* progs/hello

.PHONY: progs
progs:
	@make -C progs all

rebuild_fs:
	@$(MAKE) -C progs
	@rm -f $(FSIMAGE) fs.ustar
	@$(MAKE) all

$(BUILD)/kernel : rust_kernel startup.o $(ASMOBJ_PREFIXED) $(FSIMAGE)
	@echo "---LINKING ... ---------------"
	$(VERBOSE) ld $(LDFLAGS) -T $(LINKER_SCRIPT) -o $@ $(BUILD)/startup.o $(ASMOBJ_PREFIXED) $(RUST_OBJECT) $(FSIMAGE)

# Note: this target works when the VPATH is set correctly
$(BUILD)/_%.o : %.s | $(BUILD)
	@echo "o  ASM OBJ	$@"
	@if test \( ! \( -d $(@D) \) \) ;then mkdir -p $(@D);fi
	$(VERBOSE) nasm $(NASMFLAGS) -o $@ $<

rust_kernel:
	@echo "---BUILDING RUST KERNEL-------"
	$(VERBOSE) RUSTFLAGS="$(RUSTC_FLAGS)" cargo build --target $(CARGO_XBUILD_TARGET) $(CARGO_XBUILD_FLAGS)

# compile the assembly source
# TODO make this arch dependent
startup.o: boot/startup-$(ARCH).s | $(BUILD)
	@echo "o  ASM OBJ	$@"
	@if test \( ! \( -d $(@D) \) \) ;then mkdir -p $(@D);fi
	$(VERBOSE) nasm $(NASMFLAGS) -o $(BUILD)/startup.o boot/startup-$(ARCH).s

.PHONY: $(BUILD)
$(BUILD):
	@mkdir -p $@

.PHONY: check
check:
	@echo "---CHECKING FORMAT AND LINTS--"
	@cargo +nightly fmt --all -- --check -l
	@cargo +nightly clippy --target defs/x86_64-rustubs.json

clean:
	cargo clean
	rm -f startup.o kernel bootdisk.iso
	rm -f isofiles/boot/kernel
	rm -f build/*
	rm -f $(FSIMAGE) fs.ustar

qemu: bootdisk.iso
	qemu-system-x86_64 -drive file=./bootdisk.iso,format=raw -k en-us -serial mon:stdio

gdb:
	gdb -x /tmp/gdbcommands.$(shell id -u) build/kernel

qemu-gdb: bootdisk.iso
	@echo "target remote localhost:9876" > /tmp/gdbcommands.$(shell id -u)
	@qemu-system-x86_64 -drive file=bootdisk.iso,format=raw -k en-us -S -gdb tcp::9876 -serial mon:stdio

.PHONY: rust-docs docs
rust-docs:
	cargo doc --document-private-items --all-features --no-deps --target $(CARGO_XBUILD_TARGET)

docs:
	cargo doc --document-private-items --all-features --no-deps --target $(CARGO_XBUILD_TARGET) --open

test:
	@echo "---BUILD DIR---"
	@echo $(BUILD)
	@echo "---ASM SRC---"
	@echo $(ASM_SOURCES)
	@echo "---ASM OBJ---"
	@echo $(ASM_OBJECTS)
	@echo "---ASM OBJ PREFIXED"
	@echo $(ASMOBJ_PREFIXED)

.PHONY: clean qemu test
