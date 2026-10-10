mod adc;
pub mod epoch;
pub mod injected;
mod interrupt;
mod pac;
mod pac_instance;
pub mod slow_vref;
mod state;
#[cfg(feature = "adc-timing")]
pub mod timing;

pub use adc::*;
pub use interrupt::SingleInterruptHandler;
