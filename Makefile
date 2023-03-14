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

# verbose for testing; VERBOSE=@ to turn off..
VERBOSE=@
BUILD = build
ARCH = x86_64
ASM_SOURCES = $(shell find ./src -name "*.s")
ASM_OBJECTS = $(patsubst %.s,_%.o, $(notdir $(ASM_SOURCES)))
ASM = nasm
ASMOBJFORMAT = elf64

# I don't like this style... but what can I do?
ASMOBJ_PREFIXED = $(addprefix $(BUILD)/,$(ASM_OBJECTS))
# Setting directories to look for missing source files
VPATH = $(sort $(dir $(ASM_SOURCES)))

LINKER_SCRIPT = ./src/arch/$(ARCH)/linker.ld

# include --release flag to build optimized code
CARGO_XBUILD_FLAG = 
ifneq ($(CARGO_XBUILD_FLAG), --release)
	RUST_BUILD = debug
else
	RUST_BUILD = release
endif
		
RUST_OBJECT = target/$(ARCH)-rustubs/$(RUST_BUILD)/librustubs.a

all: bootdisk.iso

bootdisk.iso : kernel
	$(VERBOSE) cp kernel isofiles/boot/
	$(VERBOSE) grub-mkrescue /usr/lib/grub/i386-pc -o bootdisk.iso isofiles

# Note: explicitly tell the linker to use startup: as the entry point (we have no main here)
kernel : rust_kernel startup.o $(ASMOBJ_PREFIXED)
	$(VERBOSE) ld -static -e startup -T $(LINKER_SCRIPT) -o ./kernel $(BUILD)/startup.o $(ASMOBJ_PREFIXED) $(RUST_OBJECT)

# Note: this target works when the VPATH is set correctly
$(BUILD)/_%.o : %.s
	@echo "ASM		$@"
	@if test \( ! \( -d $(@D) \) \) ;then mkdir -p $(@D);fi
	$(VERBOSE) $(ASM) -f $(ASMOBJFORMAT) -o $@ $<


# install xbuild first. (cargo install xbuild)
# Compile the rust part: note that the the cargo crate is of type [staticlib], if you don't 
# define this, the linker will have troubles, especially when we use a "no_std" build
rust_kernel:
	 cargo xbuild --target $(ARCH)-rustubs.json $(CARGO_XBUILD_FLAG)

# need nasm
startup.o:
	@if test \( ! \( -d $(@D) \) \) ;then mkdir -p $(@D);fi
	nasm -f elf64 -o $(BUILD)/startup.o startup.s

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
