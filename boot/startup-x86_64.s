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
[GLOBAL mb_magic]
[GLOBAL mb_info_addr]
; functions from other parts of rustubs
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
	cld
	cli
	; with multiboot specs, grub initialzes the registers:
	; EAX: magic value 0x2BADB002
	; EBX: 32-bit physical address of the multiboot information struct
	; we store them in global variables for future uses in rust code.
	mov	   dword [mb_magic], eax
	mov	   dword [mb_info_addr], ebx
	; setup GDT by loading GDT descriptor
	; see docs/x86_gdt.txt
	lgdt   [gdt_80]
	; use the 3rd gdt entry for protected mode segmentations
	mov    eax, 3 * 0x8
	mov    ds, ax
	mov    es, ax
	mov    fs, ax
	mov    gs, ax

	; define stack
	mov    ss, ax
	mov    esp, init_stack+STACKSIZE

init_longmode:
	; activate address extension (PAE)
	mov    eax, cr4
	or	   eax, 1 << 5
	mov    cr4, eax

setup_paging:
	; Provisional identical page mapping, using 1G huge page, therefore only 2
	; table levels needed. see docs/x86_paging.txt

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
	; select EFER (Extended Feature Enable Register)
	mov    ecx, 0x0C0000080
	rdmsr
	or	   eax, 1 << 8 ; LME (Long Mode Enable)
	wrmsr
	; activate paging
	mov    eax, cr0
	or	   eax, 1 << 31
	mov    cr0, eax

	; use the 2nd gdt entry (see definition below)
	; jump to 64-bit code segment -> full activation of Long Mode
	jmp    2 * 0x8 : longmode_start

[BITS 64]
	;	system start, part 2 (in 64-bit Long Mode)
longmode_start:
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

gdt:
	; see docs/x86_gdt.txt

	; GDT[0] should always be NULL descriptor
	dw	0,0,0,0

	; 32-bit code segment descriptor
	; limit=0xFFFF, base=0
	; Types: P|Ring0|Code/Data|Exec|NonConforming|Readable|NotAccessed
	; Flags: 4K|32-bit|Not Long Mode
	dw	0xFFFF
	dw	0x0000
	dw	0x9A00
	dw	0x00CF

	; 64-bit code segment descriptor
	; limit=0xFFFF, base=0
	; Types: P|Ring0|Code/Data|Exec|NonConforming|Readable|NotAccessed
	; Flags: 4K|-|LongMode|-
	dw	0xFFFF
	dw	0x0000
	dw	0x9A00
	dw	0x00AF

	; data segment descriptor
	; limit=0xFFFF, base=0
	; Types:  Present|Ring0|Code/Data|NoExec|GrowUp|Writable|NotAccessed
	; Flags:  4K|32-bit|Not Long Mode
	dw	0xFFFF
	dw	0x0000
	dw	0x9200
	dw	0x00CF

gdt_80:
	dw	4*8 - 1   ; GDT limit=24, 4 GDT entries - 1
	dq	gdt		  ; GDT address

; multiboot info
mb_magic:
	dd 	0x00000000
mb_info_addr:
	dd 	0x00000000

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
