#![no_std]

mod board;
mod boards;
#[cfg(feature = "cap-drv8301")]
mod drv8301;
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
#[cfg(feature = "board")]
pub use boards::{BOARD_ID, limits};
#[cfg(feature = "cap-drv8301")]
pub use drv8301::Drv8301;
#[cfg(any(feature = "cap-voltage-filter", feature = "cap-current-filter"))]
pub use filter::SenseFilter;
#[cfg(feature = "board")]
pub use limits::*;

pub type FlashError = embassy_stm32::flash::Error;
