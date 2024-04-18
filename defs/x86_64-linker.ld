SECTIONS
{
    PROVIDE (___KERNEL_START__ = .);
    . = 0x100000;     /* system's start address */

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

	.reserved : {
		*(".reserved")
		*(".reserved.*")
	}

	/*
	 * basically the same as BSS, but I want some flexibility and I don't care
	 * for zeroing because it's explicitly overwritten anyways. I KNOW WHAT I'M
	 * DOING! An example is the idt.
	 */
	.reserved_0 (NOLOAD) : {
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
        PROVIDE (___BSS_START__ = .);
        *(".bss")
        *(".bss.*")
        PROVIDE (___BSS_END__ = .);
    }

    /* global page table for 64-bit long mode */
    .global_pagetable ALIGN(4096) (NOLOAD) :
    {
        *(".global_pagetable")
    }

    . = ALIGN(4096);
    PROVIDE (___KERNEL_END__ = .);
}
