; THis code is copied from the OSC lab project OOStuBS @ TU Dresden

;******************************************************************************
;* Operating-System Construction                                              *
;*----------------------------------------------------------------------------*
;*                                                                            *
;*                        S T A R T U P . A S M                               *
;*                                                                            *
;*----------------------------------------------------------------------------*
;* The 'startup' function is the entry point for the whole system. Switching  *
;* to 32-bit Protected Mode has already been done (by a boot loader that runs *
;* before). Here we prepare everything to be able to start running rust code  *
;* in 64-bit Long Mode as quickly as possible.                                *
;******************************************************************************

;
;   Constants
;

; stack for the main function (renamed to _entry())
STACKSIZE: equ 65536

; 512 GB maximum RAM size for page table
; DON'T MODIFY THIS UNLESS YOU UPDATE THE setup_paging accordingly
MAX_MEM: equ 512

; Multiboot constants
MULTIBOOT_PAGE_ALIGN     equ   1<<0
MULTIBOOT_MEMORY_INFO    equ   1<<1

; magic number for Multiboot
MULTIBOOT_HEADER_MAGIC   equ   0x1badb002

; Multiboot flags (ELF specific!)
MULTIBOOT_HEADER_FLAGS   equ   MULTIBOOT_PAGE_ALIGN | MULTIBOOT_MEMORY_INFO
MULTIBOOT_HEADER_CHKSUM  equ   -(MULTIBOOT_HEADER_MAGIC + MULTIBOOT_HEADER_FLAGS)
MULTIBOOT_EAX_MAGIC      equ   0x2badb002


;
;   System
;

; exported symbols
[GLOBAL startup]
[GLOBAL idt]
[GLOBAL pml4]
[GLOBAL pdp]

; functions from the C parts of the system
[EXTERN _entry]
[EXTERN guardian]

; addresses provided by the compiler
[EXTERN ___BSS_START__]
[EXTERN ___BSS_END__]

[SECTION .text]

;
;   system start, part 1 (in 32-bit Protected Mode)
;
;   GDT and page-table initialization, and switch to 64-bit Long Mode
;

[BITS 32]

	jmp    startup  ; jump over Multiboot header
	align  4        ; 32-bit alignment for GRUB

;
;   Multiboot header for starting with GRUB or QEMU (w/o BIOS)
;

	dd MULTIBOOT_HEADER_MAGIC
	dd MULTIBOOT_HEADER_FLAGS
	dd MULTIBOOT_HEADER_CHKSUM
	dd 0 ; header_addr (gets ignored)
	dd 0 ; load_addr (gets ignored)
	dd 0 ; load_end_addr (gets ignored)
	dd 0 ; bss_end_addr (gets ignored)
	dd 0 ; entry_addr (gets ignored)
	dd 0 ; mode_type (gets ignored)
	dd 0 ; width (gets ignored)
	dd 0 ; height (gets ignored)
	dd 0 ; depth (gets ignored)

;
;  GRUB entry point
;

startup:
	cld              ; GCC-compiled code expects the direction flag to be 0
	cli              ; disable interrupts
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
	or     eax, 1 << 5
	mov    cr4, eax

	; create page table (mandatory)
	call   setup_paging

	; activate Long Mode (for now in compatibility mode)
	mov    ecx, 0x0C0000080 ; select EFER (Extended Feature Enable Register)
	rdmsr
	or     eax, 1 << 8 ; LME (Long Mode Enable)
	wrmsr

	; activate paging
	mov    eax, cr0
	or     eax, 1 << 31
	mov    cr0, eax

	; jump to 64-bit code segment -> full activation of Long Mode
	jmp    2 * 0x8 : longmode_start

;
; Provisional identical page mapping, using 1G huge page (therefore only 2 table
; levels needed)
;

setup_paging:
	; PML4 (Page Map Level 4 / 1st level)
	mov    eax, pdp
	or     eax, 0xf
	mov    dword [pml4+0], eax
	mov    dword [pml4+4], 0
	; PDPE flags
	mov    eax, 0x0 | 0x87    ; start-address bytes bit [30:31] + flags
	mov    ebx, 0             ; start-address bytes bit [32:38]
	mov    ecx, 0
fill_tables2:
	; fill one single PDP table, with 1G pages, 512 PDPE maps to 512 GB
	cmp    ecx, MAX_MEM
	je     fill_tables2_done
	mov    dword [pdp + 8*ecx + 0], eax ; low bytes
	mov    dword [pdp + 8*ecx + 4], ebx ; high bytes
	add    eax, 0x40000000      		; 1G per page
	adc    ebx, 0             ; overflow? -> increment higher-order half of the address
	inc    ecx
	ja     fill_tables2
fill_tables2_done:
	; set base pointer to PML4
	mov    eax, pml4
	mov    cr3, eax
	ret

;
;   system start, part 2 (in 64-bit Long Mode)
;
;   This code clears the BSS segment and initializes IDT and PICs. Then the
;   constructors of global C++ objects are called, and finally _entry() is run.
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

	; initialize IDT and PICs
	call   setup_idt

	fninit         ; activate FPU

	; init SSE
	; NOTE: must NOT use sse target features for rust compiler, if sse not enabled here.
	;mov rax, cr0
	;and rax, ~(1 << 2)	;clear coprocessor emulation CR0.EM
	;or rax, 1 << 1		;set coprocessor monitoring  CR0.MP
	;mov cr0, rax
	;mov rax, cr4
	;or rax, 3 << 9		;set CR4.OSFXSR and CR4.OSXMMEXCPT at the same time
	;mov cr4, rax

	call   _entry  ; call the OS kernel's C / C++ part
	cli            ; Usually we should not get here.
	hlt

;
;   Interrupt handling
;

; template for header for each interrupt-handling routine
; TODO: vectors should have their dedicated place
%macro wrapper 1
wrapper_%1:
	push   rbp
	mov    rbp, rsp
	push   rax
	mov    al, %1
	jmp    wrapper_body
%endmacro

; automatic generation of 256 interrupt-handling routines, based on above macro
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

	; pass interrupt number as the first parameter
	mov    rdi, rax
    ; call the interrupt handler wrapper here.
    ; TODO implement it in rust then uncomment the line
    call   guardian

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

;
; Relocating of IDT entries and setting IDTR
;

setup_idt:
	mov    rax, wrapper_0

	; bits 0..15 -> ax, 16..31 -> bx, 32..64 -> edx
	mov    rbx, rax
	mov    rdx, rax
	shr    rdx, 32
	shr    rbx, 16

	mov    r10, idt   ; pointer to the actual interrupt gate
	mov    rcx, 255   ; counter
.loop:
	add    [r10+0], ax
	adc    [r10+6], bx
	adc    [r10+8], edx
	add    r10, 16
	dec    rcx
	jge    .loop

	lidt   [idt_descr]
	ret


[SECTION .data]

;
; Segment descriptors
;

gdt:
	dw  0,0,0,0   ; NULL descriptor

	; 32-bit code segment descriptor
	dw  0xFFFF    ; 4Gb - (0x100000*0x1000 = 4Gb)
	dw  0x0000    ; base address=0
	dw  0x9A00    ; code read/exec
	dw  0x00CF    ; granularity=4096, 386 (+5th nibble of limit)

	; 64-bit code segment descriptor
	dw  0xFFFF    ; 4Gb - (0x100000*0x1000 = 4Gb)
	dw  0x0000    ; base address=0
	dw  0x9A00    ; code read/exec
	dw  0x00AF    ; granularity=4096, 386 (+5th nibble of limit), Long-Mode

	; data segment descriptor
	dw  0xFFFF    ; 4Gb - (0x100000*0x1000 = 4Gb)
	dw  0x0000    ; base address=0
	dw  0x9200    ; data read/write
	dw  0x00CF    ; granularity=4096, 386 (+5th nibble of limit)

gdt_80:
	dw  4*8 - 1   ; GDT limit=24, 4 GDT entries - 1
	dq  gdt       ; GDT address

;
; Interrupt descriptor table with 256 entries
;

idt:
%macro idt_entry 1
	dw  (wrapper_%1 - wrapper_0) & 0xffff ; offset 0 .. 15
	dw  0x0000 | 0x8 * 2 ; selector points to 64-bit code segment selector (GDT)
	dw  0x8e00 ; 8 -> interrupt is present, e -> 80386 32-bit interrupt gate
	dw  ((wrapper_%1 - wrapper_0) & 0xffff0000) >> 16 ; offset 16 .. 31
	dd  ((wrapper_%1 - wrapper_0) & 0xffffffff00000000) >> 32 ; offset 32..63
	dd  0x00000000 ; reserved
%endmacro

%assign i 0
%rep 256
idt_entry i
%assign i i+1
%endrep

idt_descr:
	dw  256*8 - 1    ; 256 entries
	dq idt

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
