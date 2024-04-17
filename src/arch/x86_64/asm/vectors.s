; vectors.s - idt for x86_64

[GLOBAL idt]
[GLOBAL idt_descr]
[GLOBAL vectors_start]
[EXTERN interrupt_gate]

[SECTION .data.idt]
;
; Interrupt descriptor table with 256 entries
; TODO: use a interrupt stack instead of the current stack.
;
idt:
%macro idt_entry 1
	dw	(wrapper_%1 - wrapper_0) & 0xffff ; offset 0 .. 15
	dw	0x0000 | 0x8 * 2 ; selector points to 64-bit code segment selector (GDT)
	dw	0x8e00 ; 8 -> interrupt is present, e -> 80386 32-bit interrupt gate
	dw	((wrapper_%1 - wrapper_0) & 0xffff0000) >> 16 ; offset 16 .. 31
	dd	((wrapper_%1 - wrapper_0) & 0xffffffff00000000) >> 32 ; offset 32..63
	dd	0x00000000 ; reserved
%endmacro

%assign i 0
%rep 256
idt_entry i
%assign i i+1
%endrep

idt_descr:
	dw	256*8 - 1	 ; 256 entries
	dq idt

; template for header for each interrupt-handling routine
%macro wrapper 1
wrapper_%1:
	push   rbp
	mov    rbp, rsp
	push   rax
	mov    al, %1
	jmp    wrapper_body
%endmacro

; automatic generation of 256 interrupt-handling routines, based on above macro

[SECTION .data.vectors]
vectors_start:
%assign i 0
%rep 256
wrapper i
%assign i i+1
%endrep

; common handler body
wrapper_body:
	; GCC expects the direction flag to be 0
	cld
	; save volatile registers
	push   rcx
	push   rdx
	push   rdi
	push   rsi
	push   r8
	push   r9
	push   r10
	push   r11

	; the generated wrapper only gives us 8 bits, mask the rest
	and    rax, 0xff
	; call the interrupt handling code with interrupt number as parameter
	mov    rdi, rax
	call   interrupt_gate

	; restore volatile registers
	pop    r11
	pop    r10
	pop    r9
	pop    r8
	pop    rsi
	pop    rdi
	pop    rdx
	pop    rcx

	; ... also those from the wrapper
	pop    rax
	pop    rbp

	; done
	iretq
