//! machine level abstractions for architecture independent devices.
//! FIXME: still having some x86 coupling.
pub mod cgascr;
pub mod device_io;
pub mod interrupt;
pub mod key;
pub mod keyctrl;
pub mod mem;
pub mod multiboot;
pub mod serial;

// TODO: this module *should* be arch independent.
