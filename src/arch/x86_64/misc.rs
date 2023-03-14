// wrappers for misc. architectural code
// before they find better place to go.
// asm code goes to asm/misc.s

extern "C" {
    fn _delay();
}

#[inline(always)]

// delays for several cycles. Used to fill sequantial IO commands
// (for devices to react)
pub fn delay() {
    unsafe {
        _delay();
    }                
}
