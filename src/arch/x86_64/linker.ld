SECTIONS
{
   . = 0x100000;     /* system's start address */

   .text :
   {
      *(".text")
      *(".text.*")
      *(".text$")
      *(".init")
      *(".fini")
      *(".gnu.linkonce.*")
   }

   .init_array :
   {
      PROVIDE_HIDDEN (__init_array_start = .);
      KEEP (*(SORT(.init_array.*)))
      KEEP (*(.init_array))
      KEEP (*(".ctors"))
      KEEP (*(".ctor"))
      PROVIDE_HIDDEN (__init_array_end = .);
   }

   .fini_array :
   {
      PROVIDE_HIDDEN (__fini_array_start = .);
      KEEP (*(SORT(.fini_array.*)))
      KEEP (*(.fini_array))
      KEEP (*(".dtors"))
      KEEP (*(".dtor"))
      PROVIDE_HIDDEN (__fini_array_end = .);
   }

   .data :
   {
      *(".data")
      *(".data$")
      *(".rodata")
      *(".rodata.*")
      *(".got")
      *(".got.plt")
      *(".eh_frame")
      *(".eh_fram")
      *(".jcr")
      *(".note.*")
   }

   .bss :
   {
      ___BSS_START__ = .;
      *(".bss")
      *(".bss.*")
      ___BSS_END__ = .;
   }

   /* global page table for 64-bit long mode */
   .global_pagetable ALIGN(4096) (NOLOAD) :
   {
      *(".global_pagetable")
   }

/*
   /DISCARD/ :
   {
      *(".note")
      *(".comment")
      *(".debug_line")
      *(".debug_info")
      *(".debug_abbrev")
      *(".debug_aranges")
   }
*/
	. = ALIGN(4096);
	PROVIDE (___KERNEL_END__ = .);
}
