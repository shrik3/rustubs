# this is an prototype makefile with hardcodings..
# Forgive the ugly code, but this make more sense
# for those (me included) who are not sure about 
# the building process.
# TODO reorganize...
#

all: bootdisk.iso


bootdisk.iso : kernel
	cp kernel isofiles/boot/
	grub-mkrescue /usr/lib/grub/i386-pc -o bootdisk.iso isofiles

# Link the rust library against the objects from asm code (currently only the startup.o),
# later we'll use wildcards
kernel : rust_kernel startup.o
	ld -static -e startup -T ./src/arch/x86_64/linker.ld -o ./kernel startup.o target/x86_64-rustubs/debug/librustubs.a

# install xbuild first. (cargo install xbuild)
# Compile the rust part: note that the the cargo crate is of type [staticlib], if you don't 
# define this, the linker will have troubles, especially when we use a "no_std" build
rust_kernel:
	 cargo xbuild --target x86_64-rustubs.json

# need nasm
startup.o:
	nasm -f elf64 -o startup.o src/arch/x86_64/asm/startup.s

clean:
	cargo clean
	rm -f bootdisk.iso
	rm -f startup.o
	rm -f kernel
	rm -f isofiles/boot/kernel
	
qemu: bootdisk.iso
	qemu-system-x86_64 -drive file=./bootdisk.iso,format=raw -k en-us
