mod common;
pub mod error;
mod manager;
pub mod serial;

pub use common::*;
pub use manager::InterfaceManager;

tonic::include_proto!("pyrion.v1.interface");
