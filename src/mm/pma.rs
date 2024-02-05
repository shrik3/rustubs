use core::ffi::c_void;
use crate::defs::Mem;
// this is POC code, it will be ugly
extern "C" {
    static KERNEL_END: *const c_void;
}

// pub struct PageAlloctor;

