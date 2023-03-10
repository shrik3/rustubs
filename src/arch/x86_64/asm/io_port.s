;***************************************************************************
;* Operating-System Construction                                             *
;*---------------------------------------------------------------------------*
;*                                                                           *
;*                             I O _ P O R T                                 *
;*                                                                           *
;*---------------------------------------------------------------------------*
;* The functions defined here encapsulate the machine instructions 'in' and  *
;* 'out' for class IO_Port.                                                  *
;*****************************************************************************

; EXPORTED FUNCTIONS

[GLOBAL outb]
[GLOBAL outw]
[GLOBAL inb]
[GLOBAL inw]

; FUNCTION IMPLEMENTATIONS

[SECTION .text]

; OUTB: Byte-wise output via an I/O port.
;
;       C prototype: void outb (int port, int value);

outb:
	push   rbp
	mov    rbp, rsp
	mov    rdx, rdi
	mov    rax, rsi
	out    dx, al
	pop    rbp
	ret

; OUTW: Word-wise output via an I/O port.
;
;       C prototype: void outw (int port, int value);

outw:
	push   rbp
	mov    rbp, rsp
	mov    rdx, rdi
	mov    rax, rsi
	out    dx, ax
	pop    rbp
	ret

; INB: Byte-wise input via an I/O port.
;
;      C prototype: unsigned char inb (int port);

inb:
	push   rbp
	mov    rbp, rsp
	mov    rdx, rdi
	in     al, dx
	pop    rbp
	ret

; INW: Word-wise input via an I/O port.
;
;      C prototype: unsigned short inw (int port);

inw:
	push   rbp
	mov    rbp, rsp
	mov    rdx, rdi
	in     ax, dx
	pop    rbp
	ret
