mod adc;
mod communication;
mod leds;
mod safety;
mod shaft_position;
#[cfg(feature = "cap-uart")]
mod uart;
mod usb;

pub use adc::task_adc;
pub use communication::task_communication;
pub use communication::{COMMAND_CHANNEL, EVENT_CHANNEL};
pub use leds::task_leds;
#[cfg(feature = "cap-external-i2c")]
pub use shaft_position::task_shaft_position;
#[cfg(feature = "cap-uart")]
pub use uart::task_uart;
pub use usb::task_usb;
