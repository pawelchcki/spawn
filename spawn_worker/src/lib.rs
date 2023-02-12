pub(crate) const TRAMPOLINE_BIN: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/trampoline.bin"));

#[cfg(target_family = "unix")]
mod unix;
#[cfg(target_family = "unix")]
pub use unix::*;
