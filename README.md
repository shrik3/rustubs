[![builds.sr.ht status](https://builds.sr.ht/~shrik3/rustubs/commits/master/x86_64.yml.svg)](https://builds.sr.ht/~shrik3/rustubs/commits/master/x86_64.yml?)

# License & Copyright:

The following list of files contain code copied or derived from the OOStuBS
project, an minimal operating system used in several German universities for
tutorial purposes. These code are available in public, but the Copyright status
is not clear.

```
startup.s
Makefile
arch/x86_64/linker.ld
```

This project **will adapt a copy-left license** (presumably EUPL-1.2). I'll try
to either ask for permission to distribute the copied code under a copy-left
license, or do a clean-room rewrite.

Untill then, all rights are reserverd to the original authors of OOStuBS
(although unclear). Redistribution of the above files MUST include this notice.

# The rust port of OOStuBS [WIP]

This is a toy bare metal operation system implemented in Rust. Apologies for my
shitty code, I'm a rust beginner.

The project is based on the OOStuBS, an OS exercise project used in some German
Universities. This one in perticular, is based on the TU Dresden version
(Operating System Construction), led by my Professor Dr. Horst Schirmeier.

**Status / Roadmap**
- [X] Basic code structure
- [X] Build minimal iso image
- [X] bootable using grub
- [X] Setting up CGA display, print something (hello world)
- [X] Intigrate print into rust println! etc.
- [X] Keyboard controller and input handler
- [?] Interrupt handler (WIP)
    - linked list for plugbox
    - implement plugbox
    - interrupt handler code for kbd
    - input buffer
- [ ] intrrupt sync
    - split upper/lower half of handlers
- [ ] Threading
    - stack allocator (could be trivial)
    - define context
    - implement switch/toc code
- [ ] Scheduler (single CPU)
    - DS
- [ ] Timer Interrupt
- [ ] Synchronization Primitives
    - implement waiting/wakeup
- [ ] asm! Wrappers for basic instructions

Beyond the original StuBS
- [ ] Task Descriptor structures
- [ ] Paging: PMA and paging structures
- [ ] Paging: pagefault handler
- [ ] user heap and mmap
- [ ] Upperhalf Kernel
- [ ] Address Space for each Process
- [ ] in memory FS
- [ ] user library
- [ ] syscall
- [ ] aarch64 support

## Build

Please take a look at the CI manifest:
`.builds/x86_64.yml`

**>general dependencies:**
- cargo / rustc (nightly)
- xbuild for crossbuild
- basics: nasm, make, glibc, ld etc.
- xorriso and grub (to create bootable image)
- qemu-system-x86_64 (optionly for simulation)

**before building**
- You may need to add the rust sources component by `rustup component add  rust-src`

**build and run**
- simply run `make`, you will get `bootdisk.iso`, which you can use to boot a
  bare metal
- use `make qemu` to load and test the iso image with qemu

# Remarks
**Why not projects like [blog_os](https://os.phil-opp.com/)?**
firstly, because it's my own practice. "What I can't create, I don't understand".
Secondly, the newest revision of *blog_os* can only be booted with BIOS, not
UEFI. And the complexity (e.g. the sartup.s) is hidden behind the `bootimage`,
I feel necessary to go through the painful part.

**Your code sucks**
Yes. I'm a rust beginner.

**Helper docs**

x86_64 calling conventions  
https://aaronbloomfield.github.io/pdr/book/x86-64bit-ccc-chapter.pdf

Rust inline asm  
https://rust-lang.github.io/rfcs/2873-inline-asm.html

asm Syntax : (we use nasm in assembly and .intel_syntax noprefix in rust asm)  
https://en.wikipedia.org/wiki/X86_assembly_language#Syntax

naming conventions  
https://rust-lang.github.io/api-guidelines/naming.html

Makefile Cheatsheet:  
https://devhints.io/makefile

AT Keyboard Controller:  
https://homepages.cwi.nl/~aeb/linux/kbd/scancodes-8.html

PS/2 Keyboard Controller:  
https://wiki.osdev.org/PS/2_Keyboard

TU Dresden OSC labs (the baseline for this project):  
https://tu-dresden.de/ing/informatik/sya/professur-fuer-betriebssysteme/studium/vorlesungen/betriebssystembau/lab-tasks
