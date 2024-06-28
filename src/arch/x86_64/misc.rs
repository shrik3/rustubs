//! wrappers for misc. architectural code
//! before they find better place to go.
//! asm code goes to asm/misc.s

extern "C" {
	fn _delay();
}

/// delays for several cycles. Used to fill sequantial IO commands (for devices
/// to react). This does literally nothing: call an empty function and return
#[inline(always)]
pub fn delay() { unsafe { _delay() }; }
