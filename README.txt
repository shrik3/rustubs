# The rust port of OOStuBS [WIP]

This is a toy bare metal operation system implemented in Rust. Apologies for my
shitty code, I'm a rust beginner.

The project is based on the OOStuBS, an OS exercise project used in some German
Universities. This one in perticular, is based on the TU Dresden version
(Operating System Construction), led by my Professor Dr. Horst Schirmeier.


**Status**
[X] - Basic code structure
[X] - Build minimal iso image
[X] - bootable using grub
[X] - Setting up CGA display, print something (hello world)
[X] - Provide "printf" support
[ ] - Keyboard controller and input handler
[ ] - Interrupt handler
[ ] - Timer Interrupt
[ ] - Threading
[ ] - Scheduler
[ ] - Synchronization Primitives

**Dependencies**
- cargo / rustc (nightly)
- xbuild for crossbuild
- basics: nasm, make, glibc, ld etc.
- xorriso and grub (to create bootable image)
- qemu-system-x86_64 (optionly for simulation)

**Before building**
- You may need to add the rust sources component by `rustup component add  rust-src`

**How to build**
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

