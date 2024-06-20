global _start
section .text
_start:
	mov rdi, 0xffff8000000b8640,
	mov word [rdi], 0x4e48 ; H
	add rdi, 0x2
	mov word [rdi], 0x4e65 ; e
	add rdi, 0x2
	mov word [rdi], 0x4e6c ; l
	add rdi, 0x2
	mov word [rdi], 0x4e6c ; l
	add rdi, 0x2
	mov word [rdi], 0x4e6f ; o
	add rdi, 0x2
	mov word [rdi], 0x4e2c ; ,
	add rdi, 0x2
	mov word [rdi], 0x4e20 ;
	add rdi, 0x2
	mov word [rdi], 0x4e77 ; w
	add rdi, 0x2
	mov word [rdi], 0x4e6f ; o
	add rdi, 0x2
	mov word [rdi], 0x4e72 ; r
	add rdi, 0x2
	mov word [rdi], 0x4e6c ; l
	add rdi, 0x2
	mov word [rdi], 0x4e64 ; d
	add rdi, 0x2
	mov word [rdi], 0x4e21 ; !
