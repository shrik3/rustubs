# this is an prototype makefile with hardcodings..
# Forgive the ugly code, but this make more sense
# for those (me included) who are not sure about
# the building process.
# TODO reorganize...
# TODO add dependencies (.d) if necessary but I don't think so...
#	 the librustubs is cargo self-contained), others are asm code,
#	 for which dep files are not needed
#	 And .. I don't think I'll add c/c++ files to this project..
# TODO replace hardcoded values with variables
# TODO there can be more options of grub-mkrescue
# TODO put the startup.s elsewhere (I don't like it in the root dir)
# TODO maybe put the bootdisk.iso in the build dir too ..

# verbose for testing; VERBOSE=@ to turn off..
VERBOSE=@
BUILD = build
ARCH = x86_64
ASM = nasm
ASMOBJFORMAT = elf64
ASMFLAGS = -w-zeroing
LINKER_SCRIPT = ./defs/$(ARCH)-linker.ld
CARGO_XBUILD_TARGET = ./defs/$(ARCH)-rustubs.json
CARGO_XBUILD_FLAGS = --release
# ---------- No need to edit below this line --------------
# ---------- If you have to, something is wrong -----------
ASM_SOURCES = $(shell find ./src -name "*.s")
ASM_OBJECTS = $(patsubst %.s,_%.o, $(notdir $(ASM_SOURCES)))
# I don't like this style... but what can I do?
ASMOBJ_PREFIXED = $(addprefix $(BUILD)/,$(ASM_OBJECTS))
# Setting directories to look for missing source files
VPATH = $(sort $(dir $(ASM_SOURCES)))


ifneq ($(filter --release,$(CARGO_XBUILD_FLAGS)),)  
    RUST_BUILD = release
else
	RUST_BUILD = debug
endif

RUST_OBJECT = target/$(ARCH)-rustubs/$(RUST_BUILD)/librustubs.a

all: bootdisk.iso

bootdisk.iso : $(BUILD)/kernel
	@echo "---BUILDING BOOTDISK IMAGE---"
	$(VERBOSE) cp $< isofiles/boot/
	$(VERBOSE) grub-mkrescue -d /usr/lib/grub/i386-pc --locales=en@piglatin --themes=none -o bootdisk.iso isofiles > /dev/null 2>&1

# Note: explicitly tell the linker to use startup: as the entry point (we have no main here)
$(BUILD)/kernel : rust_kernel startup.o $(ASMOBJ_PREFIXED)
	@echo "---LINKING ... ---"
	$(VERBOSE) ld -static -e startup -T $(LINKER_SCRIPT) -o $@ $(BUILD)/startup.o $(ASMOBJ_PREFIXED) $(RUST_OBJECT)

# Note: this target works when the VPATH is set correctly
$(BUILD)/_%.o : %.s | $(BUILD)
	@echo "---ASM		$@"
	@if test \( ! \( -d $(@D) \) \) ;then mkdir -p $(@D);fi
	$(VERBOSE) $(ASM) -f $(ASMOBJFORMAT) $(ASMFLAGS) -o $@ $<


# install xbuild first. (cargo install xbuild)
# Compile the rust part: note that the the cargo crate is of type [staticlib], if you don't
# define this, the linker will have troubles, especially when we use a "no_std" build
rust_kernel: check
	@echo "---BUILDING RUST KERNEL---"
	@cargo xbuild --target $(CARGO_XBUILD_TARGET) $(CARGO_XBUILD_FLAGS)

# need nasm
# TODO make this arch dependent
startup.o: boot/startup-$(ARCH).s | $(BUILD)
	@echo "---ASM		$@"
	@if test \( ! \( -d $(@D) \) \) ;then mkdir -p $(@D);fi
	$(VERBOSE) $(ASM) -f $(ASMOBJFORMAT) $(ASMFLAGS) -o $(BUILD)/startup.o boot/startup-$(ARCH).s

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
	qemu-system-x86_64 -drive file=./bootdisk.iso,format=raw -k en-us


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
