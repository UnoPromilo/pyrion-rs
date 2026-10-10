mod adc;
mod communication;
mod leds;
mod power_stage;
mod safety;
#[cfg(feature = "cap-external-i2c")]
mod shaft_position;
#[cfg(feature = "cap-uart")]
mod uart;
mod usb;

#[cfg(feature = "adc-timing")]
pub use adc::task_adc_timing_report;
pub use adc::{task_adc, task_slow_aux, task_slow_vref};
pub use communication::task_communication;
pub use communication::{COMMAND_CHANNEL, EVENT_CHANNEL};
pub use leds::task_leds;
pub(crate) use power_stage::task_gate_driver;
#[cfg(feature = "cap-external-i2c")]
pub use shaft_position::task_shaft_position;
#[cfg(feature = "cap-uart")]
pub use uart::task_uart;
pub use usb::task_usb;
