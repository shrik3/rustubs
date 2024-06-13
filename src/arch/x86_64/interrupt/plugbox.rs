//! Registrar of IRQ handling routines

use crate::arch::x86_64::interrupt::pit::PIT;
use crate::defs::IntNumber as INT;
use crate::machine::keyctrl::KeyboardDriver;
use crate::proc::sync::IRQGate;
use crate::proc::sync::IRQHandlerEpilogue;
use alloc::collections::BTreeMap;
use lazy_static::lazy_static;
lazy_static! {
	/// interrupt handler lookup table. For now it's built at compile time and
	/// is immutable. Later we may ... make it hot pluggable?
	pub static ref IRQ_GATE_MAP: BTreeMap<u16, IRQGate> = [
		(INT::TIMER,    PIT::get_gate()),
		(INT::KEYBOARD, KeyboardDriver::get_gate()),
	].iter().copied().collect();
}
