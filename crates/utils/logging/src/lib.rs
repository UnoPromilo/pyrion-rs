#![no_std]
#![macro_use]

#[cfg(feature = "error-register")]
pub mod error_register;
mod freq_meter;
mod macro_logs;
pub use freq_meter::FreqMeter;
