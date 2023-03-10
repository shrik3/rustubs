# this is an prototype makefile with hardcodings..
# TODO reorganize...
#

all: bootdisk.iso


bootdisk.iso : kernel
	cp kernel isofiles/boot/
	grub-mkrescue /usr/lib/grub/i386-pc -o bootdisk.iso isofiles

kernel : rust_kernel startup.o
	ld -static -e startup -T sections -o ./kernel startup.o target/x86_64_rustubs/debug/librustubs.rlib

rust_kernel:
	cargo rustc --target=x86_64_rustubs.json -- -C link-arg=-nostartfiles --emit=obj

startup.o:
	nasm -f elf64 -o startup.o src/arch/x86_64/asm/startup.s

clean:
	cargo clean
	rm bootdisk.iso
	rm startup.o
	rm system
	rm isofiles/boot/system
	
qemu: bootdisk.iso
	qemu-system-x86_64 -drive file=./bootdisk.iso,format=raw -k en-us
