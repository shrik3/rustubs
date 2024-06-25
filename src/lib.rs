#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(non_snake_case)]
#![no_std]
#![no_main]
#![feature(const_option)]
#![feature(sync_unsafe_cell)]
#![allow(clippy::fn_to_numeric_cast)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::upper_case_acronyms)]
// Some lints are created by the utterly deranged narcissistic soy devs who have
// a bad take on "clean code". They have no idea what "clean" or "ergonomic" is.
// These are the people who tell you to create a factory that creates a factory
// which could produce more factories of factory because so that you can have a
// nice abstraction of abstraction of abstractions.
#![allow(clippy::new_without_default)]
#![allow(clippy::needless_return)]
#![allow(clippy::needless_range_loop)]
// note: how can I get interior mutability (with mut ref) without
// triggering clippy?
#![allow(clippy::mut_from_ref)]
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
use defs::*;
use kthread::KThread;
use machine::multiboot;
use proc::task::Task;

#[no_mangle]
pub unsafe extern "C" fn _entry() -> ! {
	// initialize cga display
	io::set_attr(0x1f);
	io::reset_screen();
	// check mbi now. This will be later used to initilize the allocator
	assert!(multiboot::check(), "bad multiboot info from grub!");
	// promote gdt to high memory mapping
	arch::x86_64::gdt::init();
	// initialize the idt and re-program the pic. Must do this before enabling
	// irq also must initialize the idt before mm, because the later may trigger
	// page faults, which is fatal and we want to catch them during system
	// initilization. (disabling interrupts have no effect on exceptions)
	interrupt::init();
	// initialize memory manager
	mm::init();
	// point of no return: low memory can no longer be accessed after this point
	mm::drop_init_mapping();
	// initialize proc and sync primitives
	proc::init();
	// initialize interrupt timer to roughly ... 50 hz
	let _interval = interrupt::pit::PIT::set_interval(20000);
	pic_8259::allow(PicDeviceInt::KEYBOARD);
	pic_8259::allow(PicDeviceInt::TIMER);
	// interrupt should be enabled at the end
	// run kernel threads
	create_tasks();
	interrupt::interrupt_enable();
	Scheduler::kickoff();
	panic!("should not reach");
}

fn create_tasks() {
	let mut sched = GLOBAL_SCHEDULER.lock();
	sched.insert_task(Task::create_task(1, kthread::Idle::get_entry()));
	sched.insert_task(Task::create_task(2, kthread::Meeseeks::get_entry()));
	sched.insert_task(Task::create_task(3, kthread::Kshell::get_entry()));
	sched.insert_task(Task::create_task(4, kthread::Lazy::get_entry()));
}
