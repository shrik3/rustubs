; Contains code from from the OSC lab project OOStuBS @ TU Dresden

; stack for the main function (renamed to _entry())
STACKSIZE: equ 65536

; 512 GB maximum RAM size for page table
; DON'T MODIFY THIS UNLESS YOU UPDATE THE setup_paging accordingly
MAX_MEM: equ 512

; exported symbols
[GLOBAL startup]
[GLOBAL pml4]
[GLOBAL pdp]

; functions from the other parts
[EXTERN vectors_start]
[EXTERN idt]
[EXTERN idt_descr]
[EXTERN _entry]

; addresses provided by the linker
[EXTERN ___BSS_START__]
[EXTERN ___BSS_END__]

[SECTION .text]

[BITS 32]
startup:
	cld				 ; GCC-compiled code expects the direction flag to be 0
	cli				 ; disable interrupts
	lgdt   [gdt_80]  ; set new segment descriptors

	; global data segment
	mov    eax, 3 * 0x8
	; 0x8 is the length of each entry
	; these registers point to 4th entry the GDT (see also the code there)
	; in x86 long mode these are dummy pointers
	; which are not actually used in addressing. (don't use segmentation at all)
	; all the addresses are physical addresses from 0.
	mov    ds, ax
	mov    es, ax
	mov    fs, ax
	mov    gs, ax

	; define stack
	mov    ss, ax
	mov    esp, init_stack+STACKSIZE

;
;  Switch to 64-bit Long Mode
;

init_longmode:
	; activate address extension (PAE)
	mov    eax, cr4
	or	   eax, 1 << 5
	mov    cr4, eax

	; Provisional identical page mapping, using 1G huge page (therefore only 2 table
	; levels needed)
setup_paging:
	; PML4 (Page Map Level 4 / 1st level)
	mov    eax, pdp
	or	   eax, 0xf
	mov    dword [pml4+0], eax
	mov    dword [pml4+4], 0
	; PDPE flags
	mov    eax, 0x0 | 0x87	  ; start-address bytes bit [30:31] + flags
	mov    ebx, 0			  ; start-address bytes bit [32:38]
	mov    ecx, 0
fill_tables2:
	; fill one single PDP table, with 1G pages, 512 PDPE maps to 512 GB
	cmp    ecx, MAX_MEM
	je	   fill_tables2_done
	mov    dword [pdp + 8*ecx + 0], eax ; low bytes
	mov    dword [pdp + 8*ecx + 4], ebx ; high bytes
	add    eax, 0x40000000				; 1G per page
	adc    ebx, 0			  ; overflow? -> increment higher-order half of the address
	inc    ecx
	ja	   fill_tables2
fill_tables2_done:
	; set base pointer to PML4
	mov    eax, pml4
	mov    cr3, eax
activate_long_mode:
	; activate Long Mode (for now in compatibility mode)
	mov    ecx, 0x0C0000080 ; select EFER (Extended Feature Enable Register)
	rdmsr
	or	   eax, 1 << 8 ; LME (Long Mode Enable)
	wrmsr
	; activate paging
	mov    eax, cr0
	or	   eax, 1 << 31
	mov    cr0, eax

	; jump to 64-bit code segment -> full activation of Long Mode
	jmp    2 * 0x8 : longmode_start


;
;	system start, part 2 (in 64-bit Long Mode)
;	1. clear BSS
;	2. enable floating poitn unit
;	3. set up idt
;	4. (optional) enable SSE
;	5. jump to rust main code
;

longmode_start:
[BITS 64]
	; clear BSS
	mov    rdi, ___BSS_START__
clear_bss:
	mov    byte [rdi], 0
	inc    rdi
	cmp    rdi, ___BSS_END__
	jne    clear_bss
	fninit		   ; activate FPU

init_sse:
	; init SSE
	; NOTE: must NOT use sse target features for rust compiler, if sse not enabled here.
	;mov rax, cr0
	;and rax, ~(1 << 2)	;clear coprocessor emulation CR0.EM
	;or rax, 1 << 1		;set coprocessor monitoring  CR0.MP
	;mov cr0, rax
	;mov rax, cr4
	;or rax, 3 << 9		;set CR4.OSFXSR and CR4.OSXMMEXCPT at the same time
	;mov cr4, rax

	call   _entry  ; call the OS kernel's rust part.
	cli			   ; Usually we should not get here.
	hlt

[SECTION .data]

;
; Segment descriptors
;

gdt:
	dw	0,0,0,0   ; NULL descriptor

	; 32-bit code segment descriptor
	dw	0xFFFF	  ; 4Gb - (0x100000*0x1000 = 4Gb)
	dw	0x0000	  ; base address=0
	dw	0x9A00	  ; code read/exec
	dw	0x00CF	  ; granularity=4096, 386 (+5th nibble of limit)

	; 64-bit code segment descriptor
	dw	0xFFFF	  ; 4Gb - (0x100000*0x1000 = 4Gb)
	dw	0x0000	  ; base address=0
	dw	0x9A00	  ; code read/exec
	dw	0x00AF	  ; granularity=4096, 386 (+5th nibble of limit), Long-Mode

	; data segment descriptor
	dw	0xFFFF	  ; 4Gb - (0x100000*0x1000 = 4Gb)
	dw	0x0000	  ; base address=0
	dw	0x9200	  ; data read/write
	dw	0x00CF	  ; granularity=4096, 386 (+5th nibble of limit)

gdt_80:
	dw	4*8 - 1   ; GDT limit=24, 4 GDT entries - 1
	dq	gdt		  ; GDT address


[SECTION .bss]

global init_stack:data (init_stack.end - init_stack)
init_stack:
	resb STACKSIZE
.end:

[SECTION .global_pagetable]


pml4:
	resb   4096
	alignb 4096

pdp:
	resb   4096
	alignb 4096
