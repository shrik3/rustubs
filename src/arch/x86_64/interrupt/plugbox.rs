//! Registrar of IRQ handling routines

use crate::defs::IntNumber as INT;
use crate::machine::interrupt::pic_8259::PicDeviceInt;
use crate::machine::keyctrl::KeyboardDriver;
use crate::proc::sync::IRQGate;
use crate::proc::sync::L3SyncCell;
use crate::proc::sync::{IRQHandler, IRQHandlerEpilogue};
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use core::cell::Cell;
use lazy_static::lazy_static;
lazy_static! {
	/// interrupt handler lookup table. For now it's built at compile time and
	/// is immutable. Later we may ... make it how pluggable?
	pub static ref IRQ_GATE_MAP: BTreeMap<u16, IRQGate> = [
		(INT::KEYBOARD, KeyboardDriver::get_gate()),
	].iter().copied().collect();
}
