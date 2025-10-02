mod adc;
mod config;
pub mod injected;
mod pac;
mod pac_instance;
mod prescaler;
pub mod regular;
pub mod trigger_edge;
mod state;
mod interrupt;

pub use adc::*;
pub use config::*;
pub use interrupt::InterruptHandler;
