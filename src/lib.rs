#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(non_snake_case)]
#![no_std]
#![no_main]
#![feature(const_option)]
#![feature(try_with_capacity)]
#![feature(sync_unsafe_cell)]
#![feature(linked_list_retain)]
pub mod arch;
pub mod defs;
#[macro_use]
pub mod io;
pub mod black_magic;
pub mod fs;
pub mod kthread;
pub mod machine;
pub mod mm;
pub mod proc;
extern crate alloc;
use crate::proc::sched::*;
use arch::x86_64::interrupt;
use arch::x86_64::interrupt::pic_8259;
use arch::x86_64::interrupt::pic_8259::PicDeviceInt;
use core::panic::PanicInfo;
use defs::*;
use kthread::KThread;
use machine::multiboot;
use proc::sync::L3GetRef;
use proc::task::Task;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
	println!("[{}] {}", Task::current().unwrap().pid, info);
	loop {}
}

#[no_mangle]
pub extern "C" fn _entry() -> ! {
	// initialize cga display
	io::set_attr(0x1f);
	io::reset_screen();
	// check mbi now. This will be later used to initilize the allocator
	assert!(multiboot::check(), "bad multiboot info from grub!");
	// promote gdt to high memory mapping
	unsafe {
		arch::x86_64::gdt::init();
	}
	// initialize the idt and re-program the pic. Must do this before enabling
	// irq also must initialize the idt before mm, because the later may trigger
	// page faults, which is fatal and we want to catch them during system
	// initilization.
	interrupt::init();
	// initialize memory manager
	mm::init();
	// point of no return: low memory can no longer be accessed after this point
	unsafe { mm::drop_init_mapping() };
	// initialize interrupt timer to roughly ... 50 hz
	let _interval = interrupt::pit::PIT::set_interval(20000);
	pic_8259::allow(PicDeviceInt::KEYBOARD);
	pic_8259::allow(PicDeviceInt::TIMER);
	interrupt::interrupt_enable();
	// run kernel threads
	start();
	panic!("should not reach");
}

pub fn start() {
	L3_CRITICAL! {
		let sched = unsafe { GLOBAL_SCHEDULER.l3_get_ref_mut() };
		sched.insert_task(
			Task::create_task(1, kthread::Idle::get_entry())
		);

		sched.insert_task(
			Task::create_task(2, kthread::Meeseeks::get_entry())
		);

		sched.insert_task(
			Task::create_task(3, kthread::Kshell::get_entry())
		);

		sched.insert_task(
			Task::create_task(4, kthread::Lazy::get_entry())
		);
	}
	unsafe { Scheduler::kickoff() };
}

pub fn _print_init_mem_info() {
	println!(
		"[init] kernel mapped @ {:#X} - {:#X}",
		unsafe { vmap_kernel_start() },
		unsafe { vmap_kernel_end() },
	);
	println!(
		"[init] BSS mapped    @ {:#X} - {:#X}",
		bss_start(),
		bss_end()
	);
}

pub unsafe fn _test_pf() {
	// try a page fault
	use core::arch::asm;
	use core::slice;
	let name_buf = slice::from_raw_parts_mut(0xffffffffffff0000 as *mut u64, 10);
	asm!("mov [rdi], rax", in("rdi") name_buf.as_mut_ptr());
}
