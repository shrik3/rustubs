/* defs for Multiboot headers */
/* https://www.gnu.org/software/grub/manual/multiboot/multiboot.txt */
MB_MAGIC = 0x1badb002;
/* bit 0
 * all boot modules loaded along with the operating system must be
 * aligned on page (4KB) bit 1 must include mem_* structures
 */
MB_FLAGS = 0x3;
MB_CHKSUM = 0x100000000 - (MB_MAGIC + MB_FLAGS);

SECTIONS
{
	. = 0x100000;	  /* system's start address */

	.boot :
	{
		header_start = .;
		LONG(MB_MAGIC)
		LONG(MB_FLAGS)
		LONG(MB_CHKSUM)
		LONG(0)
		LONG(0)
		LONG(0)
		LONG(0)
		LONG(0)
		LONG(0)
		LONG(0)
		LONG(0)
		LONG(0)
		header_end = .;
	}

	PROVIDE (___KERNEL_PM_START__ = .);
	.text :
	{
		*(".text")
		*(".text.*")
		*(".text$")
	}

	.data :
	{
		*(".data")
		*(".data.*")
		*(".data$")
	}

	.reserved :
	{
		*(".reserved")
		*(".reserved.*")
	}

	/*
	 * basically the same as BSS, but I want some flexibility and I don't care
	 * for zeroing because it's explicitly overwritten anyways. I KNOW WHAT I'M
	 * DOING! An example is the idt.
	 */
	.reserved_0 (NOLOAD) :
	{
		*(".reserved_0")
		*(".reserved_0.*")
	}

	.rodata :
	{
		*(".rodata")
		*(".rodata$")
		*(".rodata.*")
	}

	.bss :
	{
		PROVIDE (___BSS_PM_START__ = .);
		*(".bss")
		*(".bss.*")
		PROVIDE (___BSS_PM_END__ = .);
	}

	/* global page table for 64-bit long mode */
	.global_pagetable ALIGN(4096) (NOLOAD) :
	{
		*(".global_pagetable")
	}

	. = ALIGN(4096);
	/* reserve space for a premitive stack based physical frame allocator */
	/* each frame is 4KiB in size and has a 64bit (physical) address. e.g. */
	/* for every 1 GiB physical memory we need 2 MiB space reserved for the */
	/* free stack. For a easier bootstraping we are using a fix-sized stack */
	/* array. Currently using 4GiB, therefore reserve 8MiB. */
	PROVIDE (___FREE_PAGE_STACK__ = .);
	.global_free_page_stack ALIGN(4096) (NOLOAD) :
	{
		*("..global_free_page_stack")
	}
	. = ALIGN(4096);
	PROVIDE (___KERNEL_PM_END__ = .);
}
