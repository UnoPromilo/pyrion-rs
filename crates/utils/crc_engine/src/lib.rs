#![no_std]

const CRC_POLY: u16 = 0x8005;

mod engine;
#[cfg(feature = "hardware")]
pub mod hardware;
#[cfg(feature = "software")]
pub mod software;

pub use engine::CrcEngine;
