#![no_std]
pub const MAX_PACKET_SIZE: usize = (u8::MAX as usize) + 4;

pub mod command;
pub mod decoder;
pub mod encoder;
pub mod event;
pub(crate) mod helpers;
mod packet;

pub use command::Command;
pub use event::Event;
