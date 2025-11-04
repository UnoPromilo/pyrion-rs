#![no_std]

mod adc;
mod channels_macro;
mod config;
pub mod injected;
mod interrupt;
mod pac;
mod pac_instance;
mod prescaler;
mod state;
pub mod trigger_edge;

pub use adc::*;
pub use config::*;
pub use interrupt::{SingleInterruptHandler, MultiInterruptHandler};
