#![no_std]

mod adc;
mod config;
pub mod injected;
mod pac;
mod pac_instance;
mod prescaler;
// pub mod regular; TODO enable when regular is implemented
mod interrupt;
mod state;
pub mod trigger_edge;

pub use adc::*;
pub use config::*;
pub use interrupt::InterruptHandler;
