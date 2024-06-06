; vi: ft=nasm
; vectors.s - idt for x86_64
[BITS 64]
[GLOBAL idt]
[GLOBAL idt_descr]
[GLOBAL vectors_start]
[EXTERN trap_gate]

[SECTION .data.idt]
; Interrupt descriptor table with 256 entries
; TODO: use a interrupt stack instead of the current stack.
idt:
; reserve space for 256x idt entries (16 bytes each)
	resb    16 * 256

[SECTION .data.idt_descr]
idt_descr:
	dw      256*8 - 1    ; 256 entries
	dq      idt

[SECTION .text.vectors]
%macro trap_without_err 1
align 16
vector_%1:
	push    0
	push    rax
	mov     al, %1
	jmp     vector_body
%endmacro

%macro trap_with_err 1
align 16
vector_e_%1:
	push    rax
	mov     al, %1
	jmp     vector_body
%endmacro

vectors_start:
; the first 32 are x86 exceptions / traps
trap_without_err        0       ; Div By Zero
trap_without_err        1       ; Debug
trap_without_err        2       ; NMI
trap_without_err        3       ; BP
trap_without_err        4       ; OF
trap_without_err        5       ; Bound Range
trap_without_err        6       ; Invalid Opcode
trap_without_err        7       ; Device N/A
trap_with_err           8       ; Double Fault
trap_without_err        9       ; Legacy (not used)
trap_with_err           10      ; Invalid TSS
trap_with_err           11      ; Segment Not Present
trap_with_err           12      ; Stack Segment Fault
trap_with_err           13      ; GPF
trap_with_err           14      ; Page Fault
trap_without_err        15      ; RESERVED
trap_without_err        16      ; x87 FP exception
trap_with_err           17      ; Alighment check
trap_without_err        18      ; Machine Check
trap_without_err        19      ; SIMD FP Exception
trap_without_err        20      ; Virtualization Exception
trap_with_err           21      ; Control Protection
trap_without_err        22      ; RESERVED
trap_without_err        23      ; RESERVED
trap_without_err        24      ; RESERVED
trap_without_err        25      ; RESERVED
trap_without_err        26      ; RESERVED
trap_without_err        27      ; RESERVED
trap_without_err        28      ; Hypervisor Injection
trap_with_err           29      ; VMM Communication
trap_with_err           30      ; Security Exception
trap_without_err        31      ; RESERVED
; 16 PIC IRQs are remapped from 32 to 47
%assign i 32
%rep 16
	trap_without_err i
	%assign i i+1
%endrep
; irqs from 48 are not valid, we define one extra vector for all of them
trap_without_err        48      ; INVALID

; common handler body
vector_body:
	; GCC expects the direction flag to be 0
	cld
	; save volatile registers
	push    rcx
	push    rdx
	push    rdi
	push    rsi
	push    r8
	push    r9
	push    r10
	push    r11

	; the generated wrapper only gives us 8 bits, mask the rest
	and     rax, 0xff
	; the first parameter is the interrupt (exception) number
	mov     rdi, rax
	; the second parameter is a pointer to the trap frame
	mov     rsi, rsp
	; For a long jump, we need to put the (large) address in an register
	; here reusing one of the caller clobbered regs (pushed above)
	mov     r11, trap_gate
	call    r11

	; restore volatile registers
	pop     r11
	pop     r10
	pop     r9
	pop     r8
	pop     rsi
	pop     rdi
	pop     rdx
	pop     rcx

	pop     rax
	; "pop" the error code
	add     rsp, 8
	; done
	iretq
