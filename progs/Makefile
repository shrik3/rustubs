all: hello

hello: raw_hello.asm
	@echo "---USER PROG:	$@"
	@nasm -f elf64 -o hello.o $<
	@ld -o $@ hello.o

clean:
	@rm -f hello hello.o
