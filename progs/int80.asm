; https://jameshfisher.com/2018/03/10/linux-assembly-hello-world/
global _start

section .text

_start:
  mov rax, 1        ; write(
  mov rdi, 1        ;   STDOUT_FILENO,
  mov rsi, msg      ;   "Hello, world!\n",
  mov rdx, msglen   ;   sizeof("Hello, world!\n")
  int 0x80

  mov rax, 60       ; exit(
  mov rdi, 0        ;   EXIT_SUCCESS
  int 0x80

section .rodata
  msg: db "Hello, world!", 10
  msglen: equ $ - msg
