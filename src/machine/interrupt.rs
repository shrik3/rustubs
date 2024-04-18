// a "machine" level interrupt controlling interface: so that the kernel could
// enable and disable the interrupt without differentiate the architectures
// currently not in use because we are not so complicated yet. Perhaps this
// helper will deem unnecessary in the future ...
#[cfg(target_arch = "x86_64")]
pub use crate::arch::x86_64::interrupt::*;
