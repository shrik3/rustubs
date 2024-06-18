all:
	nasm -f elf64 -o hello.o raw_hello.asm
	ld -o hello hello.o

clean:
	rm -f hello hello.o
