; vectors.s - idt for x86_64

[GLOBAL idt]
[GLOBAL idt_descr]
[GLOBAL vectors_start]
[EXTERN interrupt_gate]

[SECTION .reserved_0.idt]
;
; Interrupt descriptor table with 256 entries
; TODO: use a interrupt stack instead of the current stack.

idt:
; reserve space for 256x idt entries (16 bytes each)
resb 16 * 256

[SECTION .reserved]
idt_descr:
	dw	256*8 - 1	 ; 256 entries
	dq idt

; NOTE: vectors MUST have fixed instruction length currently aligned to 16
; bytes. DO NOT modify the wrapper, instead change the wrapper_body if needed.
; if the vector has to be modified into more than 16 bytes,
; arch::x86_64:: interrupt::_idt_init() must be modified accordingly
[SECTION .reserved.vectors]
%macro wrapper 1
align 16
wrapper_%1:
	push   rbp
	mov    rbp, rsp
	push   rax
	mov    al, %1
	jmp    wrapper_body
%endmacro

; automatic generation of 256 interrupt-handling routines, based on above macro

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
