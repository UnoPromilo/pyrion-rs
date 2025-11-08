#![no_std]
use embassy_stm32::time::{Hertz, khz, mhz};

#[derive(Copy, Clone, Debug)]
pub struct UserConfig {
    pub device_name: &'static str,
    pub pwm_frequency: Hertz,
    pub onboard_i2c_frequency: Hertz,
    pub onboard_spi_frequency: Hertz,
    pub external_i2c_frequency: Hertz,
    pub external_spi_frequency: Hertz,
    pub can_bitrate: u32,
    pub fd_can_bitrate: u32,
    pub shaft_position_detector: ShaftPositionDetector,
}

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ShaftPositionDetector {
    None, // TODO is it valid?
    AS5600,
}

impl Default for UserConfig {
    // TODO load from flash
    fn default() -> Self {
        Self {
            device_name: "Pyrion V1",
            pwm_frequency: khz(40),
            onboard_i2c_frequency: khz(100),
            onboard_spi_frequency: mhz(1),
            external_i2c_frequency: khz(100),
            external_spi_frequency: mhz(1),
            can_bitrate: 250_000,
            fd_can_bitrate: 250_000,
            shaft_position_detector: ShaftPositionDetector::AS5600,
        }
    }
}

// TODO save to flash
