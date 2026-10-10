#![no_std]

mod board;
mod boards;
mod feature_guards;
#[cfg(any(feature = "cap-voltage-filter", feature = "cap-current-filter"))]
mod filter;
mod irqs;
#[cfg(feature = "board")]
mod limits;
mod mcu;
mod serial_number;

pub mod usb;

pub use board::*;
#[cfg(feature = "board-pyrion-ovo")]
pub use boards::validate_drv8301_configuration;
#[cfg(feature = "board")]
pub use boards::{AuxiliaryAdcCounts, BOARD_ID, BoardAdcFrame, FastAdcCounts, Phase, limits};
#[cfg(any(feature = "cap-voltage-filter", feature = "cap-current-filter"))]
pub use filter::SenseFilter;
#[cfg(feature = "board")]
pub use limits::*;

pub type FlashError = embassy_stm32::flash::Error;
