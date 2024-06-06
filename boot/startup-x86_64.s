; vi: ft=nasm
; Contains code from from the OSC lab project OOStuBS @ TU Dresden

; stack for the main function (renamed to _entry())
STACKSIZE: equ 65536

; 512 GB maximum RAM size for page table. Do not further increment this because
; one PML4 table (and by extension, one pdp table) covers only 512 GB address.
; And we only provision one such entry/table.
; IMPORTANT! regardless of the initial mapping size, we limit the phy memory to
; 64GB in the actual paging, so that all kernel VMAs could fit into one pml4
; entry and one pdp (level 3) table. See docs/mem_layout.txt
MAX_MEM: equ 512

; be careful with the extern and exported symbols when mapping a higher-half
; kernel: regardless where they are physically loaded
; 1) extern symbols may have 64 bit virtual addresses or values. Do not use them
;    in the 32bit part of the startup code.
; 2) if the exported (global) symbols are mapped to low (virtual) addresses,
;    they would be no longer accessable after the kernel switch to a higher half
;    mapping. This is especially true for the multiboot info data.

; Also be careful with imm width in asm instructions
; many instructions does not take 64 bit imm value. e.g. cmp. If the operand is
; an extern symbol the linker may tell you xyz "truncate to fit". In which case
; you should load the addresses or values into an register before using them

; exported symbols
[GLOBAL startup]
[GLOBAL mb_magic]
[GLOBAL mb_info_addr]
; functions from other parts of rustubs
; NOTE: this are all from 64bit code, so do not use them in 32bit assembly
[EXTERN ___BSS_START__]
[EXTERN ___BSS_END__]
[EXTERN KERNEL_OFFSET]
[EXTERN _entry]
; =============================================================================
; begin of the text secion: unlike the text* sections from the rust code the
; text here is not supposed to be relocated to an higher memory, as we can not
; use high memory until we completely set up longmode paging. Therefore we
; explicitly link the startup text section to low address. the same goes for the
; ".data32" section: they are not necessarily 32bit, the point is to confine all
; address within 4GB (32bit) range
; =============================================================================
[SECTION .text32]
[BITS 32]
startup:
	cld
	cli
	; with multiboot specs, grub initialzes the registers:
	; EAX: magic value 0x2BADB002
	; EBX: 32-bit physical address of the multiboot information struct we store
	; them in global variables for future uses in rust code. TODO place them on
	; the stack and pass as parameters to _entry
	mov     dword [mb_magic], eax
	mov     dword [mb_info_addr], ebx
	; setup GDT by loading GDT descriptor
	; see docs/x86_gdt.txt
	lgdt    [gdt_80]
	; use the 3rd gdt entry for protected mode segmentations
	mov     eax, 3 * 0x8
	mov     ds, ax
	mov     es, ax
	mov     fs, ax
	mov     gs, ax

	; define stack
	mov     ss, ax
	lea	esp, init_stack+STACKSIZE

init_longmode:
	; activate address extension (PAE)
	mov     eax, cr4
	or      eax, 1 << 5
	mov     cr4, eax

setup_paging:
	; zero out the initial page tables (3 x 4K pages in total)
	mov     edi, pml4
clear_pt:
	mov     dword [edi], 0
	add     edi, 4
	cmp     edi, pt_end
	jl      clear_pt

	; Provisional identical page mapping, using 1G huge page, therefore only 2
	; table levels needed. see docs/x86_paging.txt We provide two additional
	; mappings later in the long mode for higher half memory

	; PML4 (Page Map Level 4 / 1st level)
	; PML4 entry flag: 0xf = PRESENG | R/W | USER | Write Through
	mov     eax, pdp0
	or      eax, 0xf
	mov     dword [pml4+0], eax
	mov     dword [pml4+4], 0
	; PDPE flags 0x87 = PageSize=1G | USER | R/W | PRESENT
	mov     eax, 0x0 | 0x83    ; start-address bytes bit [30:31] + flags
	mov     ebx, 0             ; start-address bytes bit [32:38]
	mov     ecx, 0
fill_pdp0:
	; fill one single PDP table, with 1G pages, 512 PDPE maps to 512 GB
	cmp     ecx, MAX_MEM
	je      fill_pdp0_done
	mov     dword [pdp0 + 8*ecx + 0], eax ; low bytes
	mov     dword [pdp0 + 8*ecx + 4], ebx ; high bytes
	add     eax, 0x40000000
	; increment high half address on carry (overflow)
	adc     ebx, 0
	inc     ecx
	ja      fill_pdp0
fill_pdp0_done:
	; set base pointer to PML4
	mov     eax, pml4
	mov     cr3, eax
activate_long_mode:
	; activate Long Mode (for now in compatibility mode)
	; select EFER (Extended Feature Enable Register)
	mov     ecx, 0x0C0000080
	rdmsr
	or      eax, 1 << 8 ; LME (Long Mode Enable)
	wrmsr
	; activate paging
	mov     eax, cr0
	or      eax, 1 << 31
	mov     cr0, eax

	; use the 2nd gdt entry (see definition below)
	; jump to 64-bit code segment -> full activation of Long Mode
	jmp     2 * 0x8 : longmode_start


; =====================================================================
; MUST NOT USE ANY 64 BIT SYMBOLS BEFORE THIS POINT!
; may include:
; - symbols defined in 64 bit code below, if mapped to higher memory (VA)
; - all symbols exported from rust code or linker script
; =====================================================================
[BITS 64]
longmode_start:
	; now we set the pagetables for higher half memory
	; since we have Provisional paging now, why not using 64bit code?
	; the 256th entry of pml4 points to memory from 0xffff_8000_0000_0000
	mov     rax, pdp1
	; privileged, r/w, present
	or      rax, 0x3
	mov     qword [pml4+256*8], rax
	; entry 0~63 is an identical mapping with offset 0x8000_0000_0000
	; 1G Page | Privileged | R/W | PRESENT
	; TODO this should not be executable
	mov     rax, 0x0
	or      rax, 0x83
	mov     rdi, 0
fill_kvma1:
	mov     qword [pdp1 + 8*rdi], rax
	inc     rdi
	add     rax, 0x40000000
	cmp     rdi, 64
	jne     fill_kvma1
	; entry 64~127 is a hole (also some sort of protection)
	; entry 128~191 are mapping of the kernel image itself
	mov     rax, 0x0
	or      rax, 0x83
	mov     rdi, 128
fill_kvma2:
	mov     qword [pdp1 + 8*rdi], rax
	inc     rdi
	add     rax, 0x40000000
	cmp     rdi, 192
	jne     fill_kvma2
	; done :-)
	; clear BSS section for the rust code.
	mov     rdi, ___BSS_START__
	mov     rax, ___BSS_END__
clear_bss:
	; clear the BSS section before going to rust code
	; TODO: sanity check start < end, otherwise could be endless loop
	; TODO speed this up by clearing 8 bytes at once. Alignment should be taken
	; care of..
	mov     byte [rdi], 0
	inc     rdi
	cmp     rdi, rax
	jne     clear_bss
	; enable FPU
	fninit
	; NOTE: must NOT use sse target features for rust compiler, if sse not
	; enabled here.

	; shift the rsp to high memory mapping:
	mov     rax, KERNEL_OFFSET,
	or      rsp, rax
	; finally go to the rust code!
	mov     rax, _entry
	jmp     rax

	; should not reach below
	cli
	hlt

; =============================================================================
; data sections they should all have VAs identical to their PAs so we map these
; symbols differently than those generated by rust code the "data" itself
; doesn't care about 64 or 32 bit width, but we need to make sure they are not
; relocated to an address bigger then 4G (32)
; =============================================================================

[SECTION .data32]
gdt:
	; see docs/x86_gdt.txt

	; GDT[0] should always be NULL descriptor
	dw      0,0,0,0

	; 32-bit code segment descriptor
	; limit=0xFFFF, base=0
	; Types: P|Ring0|Code/Data|Exec|NonConforming|Readable|NotAccessed
	; Flags: 4K|32-bit|Not Long Mode
	dw      0xFFFF
	dw      0x0000
	dw      0x9A00
	dw      0x00CF

	; 64-bit code segment descriptor
	; limit=0xFFFF, base=0
	; Types: P|Ring0|Code/Data|Exec|NonConforming|Readable|NotAccessed
	; Flags: 4K|-|LongMode|-
	dw      0xFFFF
	dw      0x0000
	dw      0x9A00
	dw      0x00AF

	; data segment descriptor
	; limit=0xFFFF, base=0
	; Types:  Present|Ring0|Code/Data|NoExec|GrowUp|Writable|NotAccessed
	; Flags:  4K|32-bit|Not Long Mode
	dw      0xFFFF
	dw      0x0000
	dw      0x9200
	dw      0x00CF

gdt_80:
	dw      4*8 - 1   ; GDT limit=24, 4 GDT entries - 1
	dq      gdt       ; GDT address

; multiboot info
mb_magic:
	dd      0x00000000
mb_info_addr:
	dd      0x00000000

[SECTION .reserved_0.init_stack]
global init_stack:data (init_stack.end - init_stack)
init_stack:
	resb    STACKSIZE
.end:

[SECTION .global_pagetable]

; create initial page tables wrt. the memory layout we use entry 0 and 256 of
; the PML4 table the whole of the first first pdp table (512G) and entry 0, 2 of
; the second pdp table all pages here are 1 GiB huge pages
pml4:
	resb    4096
	alignb  4096
; the first PDP covers the lower 512GiB memory
pdp0:
	resb    4096
	alignb  4096
; pdp1 does the same but in higher half memory with offset
; 0xffff_8000_0000_0000, i.e. the 256th entry of pml4
pdp1:
	resb    4096
	alignb 	4096
pt_end:
; reserve 8MiB for frame alloc.
; (see linker file)
;[SECTION .global_free_page_stack]
;free_page_stack:
;       resb   8388608
;       alignb 4096
