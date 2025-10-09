#![no_std]

mod config;
mod driver;
mod error;
mod registers;

pub use config::*;
pub use driver::AS5600;
pub use error::*;
