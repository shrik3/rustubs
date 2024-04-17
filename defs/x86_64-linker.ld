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
		*(".data.idt")
		*(".data.vectors")
        *(".data")
        *(".data$")
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
