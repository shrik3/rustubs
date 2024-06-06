; vi: ft=nasm
; vectors.s - idt for x86_64
[BITS 64]
[GLOBAL context_swap]
[GLOBAL context_swap_to]
; parameters 1 (rdi) pointer to from context
; parameters 2 (rsi) pointer to to context
; struct arch_reg::Context64
context_swap:
	mov     [rdi + 8*0], rbx
	mov     [rdi + 8*1], r12
	mov     [rdi + 8*2], r13
	mov     [rdi + 8*3], r14
	mov     [rdi + 8*4], r15
	mov     [rdi + 8*5], rbp
	mov     [rdi + 8*6], rsp
	mov     rbx, [rsi + 8*0]
	mov     r12, [rsi + 8*1]
	mov     r13, [rsi + 8*2]
	mov     r14, [rsi + 8*3]
	mov     r15, [rsi + 8*4]
	mov     rbp, [rsi + 8*5]
	mov     rsp, [rsi + 8*6]
	ret

; parameters 1 (rdi) pointer to to context
context_swap_to:
	mov     rbx, [rdi + 8*0]
	mov     r12, [rdi + 8*1]
	mov     r13, [rdi + 8*2]
	mov     r14, [rdi + 8*3]
	mov     r15, [rdi + 8*4]
	mov     rbp, [rdi + 8*5]
	mov     rsp, [rdi + 8*6]
	ret
