[![builds.sr.ht status](https://builds.sr.ht/~shrik3/rustubs/commits/master/x86_64.yml.svg)](https://builds.sr.ht/~shrik3/rustubs/commits/master/x86_64.yml?)

# License & Copyright:

The following list of files contain code copied or derived from the OOStuBS
project, an minimal operating system used in several German universities for
tutorial purposes. These code are available in public, but the Copyright status
is not clear.

This project **will adapt a copy-left license** (presumably EUPL-1.2). As soon
as I do a total clean-room rewrite of the borrowed code. **As for now, all
rights are reserverd**. I'm sorry for this, but I need to stay on the safe side
before everything is sorted out.

To be specific: 
```
./boot/startup-x86_64.s
    Copyright 1998-2002  Institut für Verteilte Systeme (IVS), Otto-von-Guericke-Universität Magdeburg
    Copyright 2002-2019  Lehrstuhl für Informatik 4, Friedrich-Alexander-Universität Erlangen-Nürnberg

rest of the code:
    Copyright 2023-2024 Tianhao Wang <shrik3@mailbox.org>
```

# RuStuBS: a rust tutorial operating system inspired by OOStuBS.

This is a toy bare metal operation system implemented in Rust. Apologies for my
shitty code, I'm a rust beginner.

The project is based on the OOStuBS, an OS exercise project used in some German
Universities. This one in perticular, is based on the TU Dresden version
(Operating System Construction), led by my Professor Dr. Horst Schirmeier.

**Status / Roadmap**
- [ ] GDB support (qemu stub)
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

**general dependencies:**
- cargo / rustc (nightly)
- xbuild for crossbuild
- basics: nasm, make, glibc, ld etc.
- xorriso and grub (to create bootable image)
- qemu-system-x86_64 (optional for simulation)

**Add rust sources**
- We use `no_std` in the rust build. To use the `core` components, you need to
  add the rust sources by running e.g. `rustup component add  rust-src`

**build and run**
- simply run `make`, you will get `bootdisk.iso`, which you can use to boot a
  bare metal
- use `make qemu` to load and test the iso image with qemu

## Structure
```
.
├── boot            # early boot/startup code
├── defs            # specs for target arch, linking and compiler
├── docs            # namely
├── isofiles        # assets for the grub generated iso
├── src             # main source code
```

# Remarks
**Why not projects like [blog_os](https://os.phil-opp.com/)?**  
firstly, because it's my own practice. "What I can't create, I don't understand".
Secondly, the newest revision of *blog_os* can only be booted with BIOS, not
UEFI. And the complexity (e.g. the sartup.s) is hidden behind the `bootimage`,
I feel necessary to go through the painful part.

**Relationship w. OOStuBS**  
This project is inspired by OOStuBS. It started as a mere copy, but the path
quickly diverged.

> The third stage masks the absence of a profound reality, where the sign
> pretends to be a faithful copy, but it is a copy with no original. Signs and
> images claim to represent something real, but no representation is taking
> place and arbitrary images are merely suggested as things which they have no
> relationship to.  -- Baudrillard, Jean (1981). Simulacres et simulation

- This project DOES NOT try to complete and/or disclose the solutions to OOStuBS
  lab assignments. (There are indeed overlapping parts, but it would be the same
  amount of difficulty, if not more difficult, to read, understand and 
  translate rust code into the OOStuBS CPP code, than to read manuals and write
  CPP code yourself).
- This project DOES NOT aim to be a 1:1 port. (i.e. do the same thing but in
  rust).
- The "OO" (objekt orientiert) aspect is torn. The OOP concept creates an illusion
  that "data" and "code" magically belong to "object", which is never the case.
  I personally prefer NOT to use too much OOP in system programming.
- The "startup" code is borrowed from the OOStuBS labs @ TU Dresden. This is
  why you are still seeing "all rights reserved" instead of a copy-left license.
  I'll do the clean-room rewrite as soon as possible.

**Your code sucks**  
yes.

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

Unwinding the stack the hard way  
https://lesenechal.fr/en/linux/unwinding-the-stack-the-hard-way
