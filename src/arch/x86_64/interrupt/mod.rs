mod idt;
pub mod pic_8259;
pub mod pit;
pub mod plugbox;
use crate::arch::x86_64::arch_regs::TrapFrame;
use crate::arch::x86_64::is_int_enabled;
use crate::arch::x86_64::paging::fault;
use crate::defs::IntNumber as INT;
use crate::io::*;
use crate::machine::interrupt::plugbox::IRQ_GATE_MAP;
use crate::proc::sched::Scheduler;
use crate::proc::sync::*;
use core::arch::asm;

#[no_mangle]
#[cfg(target_arch = "x86_64")]
extern "C" fn trap_gate(nr: u16, fp: u64) {
	// cpu automatically masks interrupts so we are already in L3
	if nr < 0x20 {
		// handle exception
		handle_exception(nr, fp);
	} else {
		unsafe { handle_irq(nr) };
		// handle irq
	}

	interrupt_enable();
}

#[inline]
/// handle_irq assumes the interrupt is **disabled** when called.
/// this will also make sure interrupt is disabled when it returns
unsafe fn handle_irq(nr: u16) {
	let irq_gate = match IRQ_GATE_MAP.get(&nr) {
		None => {
			panic!("no handler for irq {}", nr);
		}
		Some(g) => g,
	};
	// execute the prologue
	irq_gate.call_prologue();
	let epi = irq_gate.get_epilogue();
	if epi.is_none() {
		// TODO? we could also take a look into the epilogue queue here when the
		// current irq doesn't have an epilogue itself. But optimistically, if
		// the epilogue queue is not empty, it's very likely someone else is
		// already working on it, so we just leave for now....
		return;
	}
	let epi = epi.unwrap();
	if !IS_L2_AVAILABLE() {
		EPILOGUE_QUEUE.l3_get_ref_mut().queue.push_back(epi);
		return;
	}
	// L2 is available, we run the epilogue now, also clear the queue before
	// return
	ENTER_L2();
	interrupt_enable();
	unsafe {
		epi.call();
	}
	// we need to clear the epilogue queue on behalf of others. Modifying the
	// epilogue is a level 3 critical section
	let mut epi: Option<EpilogueEntrant>;
	let mut done;
	loop {
		let r = irq_save();
		let rq = EPILOGUE_QUEUE.l3_get_ref_mut();
		epi = rq.queue.pop_front();
		done = rq.queue.is_empty();
		irq_restore(r);

		if let Some(e) = epi {
			assert!(is_int_enabled());
			e.call();
		}
		// This is a linearization point where we may do rescheduling
		// unlike OOStuBS, we don't do rescheduling in the epilogues. this
		// decouples the scheduler from the timer interrupt driver, also has
		// better "real-time" guarantee: rescheduling will not be delayed by
		// more than one epilogue execution; OOStuBS doesn't have the delay
		// issue because every epilogue is enqueued at most once due to the
		// limitation of having no memory management.
		LEAVE_L2();
		Scheduler::try_reschedule();
		ENTER_L2();
		// but this approach is unfair and there is no realtime guarantee (if
		// that's ever the case for us...)
		if done {
			break;
		}
	}
	// you need to make sure the interrupt is disabled at this point
	LEAVE_L2();
}

/// handles exception/faults (nr < 32);
#[inline]
fn handle_exception(nr: u16, fp: u64) {
	let frame = unsafe { &mut *(fp as *mut TrapFrame) };
	match nr {
		INT::PAGEFAULT => {
			// Pagefault
			let fault_address = fault::get_fault_addr();
			fault::page_fault_handler(frame, fault_address)
		}
		_ => {
			println!("[trap[ {:#X?}", frame);
			unsafe {
				asm!("hlt");
			}
		}
	}
}

#[inline(always)]
pub fn interrupt_enable() {
	unsafe {
		asm!("sti");
	}
}

#[inline(always)]
pub fn interrupt_disable() {
	unsafe {
		asm!("cli");
	}
}

#[inline]
/// irq_save() disables all interrupts and returns the previous state
pub fn irq_save() -> bool {
	if is_int_enabled() {
		interrupt_disable();
		return true;
	} else {
		return false;
	}
}

#[inline]
/// irq_restore only re-enable irq if was_enabled==true.
/// it will not disable irq regardless the was_enabled value. This function
/// should only be called to restore irq based on previous irq_save();
pub fn irq_restore(was_enabled: bool) {
	if was_enabled {
		interrupt_enable();
	}
}

/// initialize the idt and [pic_8259]
pub fn init() {
	// init idt
	idt::idt_init();
	// init pic
	pic_8259::_init();
}
