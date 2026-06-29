#![no_std]
#![macro_use]

#[cfg(feature = "errors")]
pub mod fault_register;
mod freq_meter;
mod macro_logs;
pub use freq_meter::FreqMeter;
