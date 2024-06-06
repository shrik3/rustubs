# this is an prototype makefile with hardcodings..
# Forgive the ugly code, but this make more sense
# for those (me included) who are not sure about
# the building process.
# TODO reorganize...
# TODO replace hardcoded values with variables
# verbose for testing; VERBOSE=@ to turn off..
VERBOSE=@
BUILD = build
ARCH = x86_64
NASMFLAGS = -w-zeroing -f elf64
LINKER_SCRIPT = ./defs/$(ARCH)-hm-linker.ld
CARGO_XBUILD_TARGET = ./defs/$(ARCH)-rustubs.json
CARGO_XBUILD_FLAGS =
RUSTC_FLAGS := -C code-model=large
# ---------- No need to edit below this line --------------
# ---------- If you have to, something is wrong -----------
LDFLAGS = -no-warn-rwx-segment -static -e startup
ASM_SOURCES = $(shell find ./src -name "*.s")
ASM_OBJECTS = $(patsubst %.s,_%.o, $(notdir $(ASM_SOURCES)))
# I don't like this style... but what can I do?
ASMOBJ_PREFIXED = $(addprefix $(BUILD)/,$(ASM_OBJECTS))
RUST_OBJECT = target/$(ARCH)-rustubs/$(RUST_BUILD)/librustubs.a
# Setting directories to look for missing source files
VPATH = $(sort $(dir $(ASM_SOURCES)))
ifneq ($(filter --release,$(CARGO_XBUILD_FLAGS)),)
    RUST_BUILD = release
else
	RUST_BUILD = debug
endif

all: bootdisk.iso

bootdisk.iso : $(BUILD)/kernel
	@echo "---BUILDING BOOTDISK IMAGE---"
	$(VERBOSE) cp $< isofiles/boot/
	$(VERBOSE) grub-mkrescue -d /usr/lib/grub/i386-pc \
		--locales=en@piglatin --themes=none \
		-o bootdisk.iso isofiles > /dev/null 2>&1

# Note: explicitly tell the linker to use startup: as the entry point (we have
# no main here)
$(BUILD)/kernel : rust_kernel startup.o $(ASMOBJ_PREFIXED)
	@echo "---LINKING ... ---"
	$(VERBOSE) ld $(LDFLAGS) -T $(LINKER_SCRIPT) -o $@ $(BUILD)/startup.o $(ASMOBJ_PREFIXED) $(RUST_OBJECT)

# Note: this target works when the VPATH is set correctly
$(BUILD)/_%.o : %.s | $(BUILD)
	@echo "---ASM		$@"
	@if test \( ! \( -d $(@D) \) \) ;then mkdir -p $(@D);fi
	$(VERBOSE) $(ASM) $(ASMFLAGS) -o $@ $<

# Compile the rust part: note that the the cargo crate is of type [staticlib],
# if you don't define this, the linker will have troubles, especially when we
# use a "no_std" build
rust_kernel: check
	@echo "---BUILDING RUST KERNEL---"
	RUSTFLAGS="$(RUSTC_FLAGS)" cargo build --target $(CARGO_XBUILD_TARGET) $(CARGO_XBUILD_FLAGS)

# compile the assembly source
# TODO make this arch dependent
startup.o: boot/startup-$(ARCH).s | $(BUILD)
	@echo "---ASM		$@"
	@if test \( ! \( -d $(@D) \) \) ;then mkdir -p $(@D);fi
	$(VERBOSE) nasm $(NASMFLAGS) -o $(BUILD)/startup.o boot/startup-$(ARCH).s

.PHONY: $(BUILD)
$(BUILD):
	@mkdir -p $@

.PHONY: check
check:
	@echo "---CHECKING FORMATTING---"
	@cargo fmt --all -- --check -l

clean:
	cargo clean
	rm -f bootdisk.iso
	rm -f startup.o
	rm -f kernel
	rm -f isofiles/boot/kernel
	rm -f build/*

qemu: bootdisk.iso
	qemu-system-x86_64 -drive file=./bootdisk.iso,format=raw -k en-us -serial mon:stdio

gdb:
	gdb -x /tmp/gdbcommands.$(shell id -u) build/kernel

qemu-gdb: bootdisk.iso
	@echo "target remote localhost:9876" > /tmp/gdbcommands.$(shell id -u)
	@qemu-system-x86_64 -drive file=bootdisk.iso,format=raw -k en-us -S -gdb tcp::9876 -serial mon:stdio

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
