use core::cell::RefCell;
#[cfg(feature = "full")]
use adc::{Adc, Continuous, Taken};
#[cfg(feature = "full")]
use crc_engine::hardware::HardwareCrcEngine;
#[cfg(feature = "full")]
use embassy_stm32::can::Can;
use embassy_stm32::flash::{Bank1Region, Bank2Region, Blocking};
use embassy_stm32::gpio::Output;
#[cfg(feature = "full")]
use embassy_stm32::i2c::I2c;
#[cfg(feature = "full")]
use embassy_stm32::mode::Async;
#[cfg(not(feature = "full"))]
use embassy_stm32::peripherals::USB;
#[cfg(feature = "full")]
use embassy_stm32::peripherals::{ADC1, ADC2, ADC3, ADC4, ADC5, TIM1, USB};
#[cfg(feature = "full")]
use embassy_stm32::spi::Spi;
#[cfg(feature = "full")]
use embassy_stm32::usart::Uart;
#[cfg(not(feature = "full"))]
use embassy_stm32::usb;
#[cfg(feature = "full")]
use embassy_stm32::{i2c, spi, usb};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::blocking_mutex::Mutex;
#[cfg(feature = "full")]
use inverter::Inverter;

pub struct Board<'a> {
    #[cfg(feature = "full")]
    pub adc: BoardAdc<'a>,
    #[cfg(feature = "full")]
    pub can: BoardCan<'a>,
    #[cfg(feature = "full")]
    pub crc: BoardCrc<'a>,
    pub flash_bank1: BoardFlashBank1<'a>,
    pub flash_bank2: BoardFlashBank2<'a>,
    #[cfg(feature = "full")]
    pub ext_i2c: BoardI2c<'a>,
    #[cfg(feature = "full")]
    pub ext_spi: BoardSpi<'a>,
    #[cfg(feature = "full")]
    pub inverter: BoardInverter<'a>,
    pub leds: BoardLeds<'a>,
    #[cfg(feature = "full")]
    pub onboard_i2c: BoardI2c<'a>,
    #[cfg(feature = "full")]
    pub onboard_spi: BoardSpi<'a>,
    #[cfg(feature = "full")]
    pub uart: BoardUart<'a>,
    pub usb: BoardUsb<'a>,
    pub serial_number: BoardSerialNumber,
}

#[cfg(feature = "full")]
pub struct BoardAdc<'a> {
    pub _adc1: Adc<'a, ADC1, Taken>,
    pub _adc2: Adc<'a, ADC2, Taken>,
    pub _adc3: Adc<'a, ADC3, Taken>,
    pub _adc4: Adc<'a, ADC4, Taken>,
    pub _adc5: Adc<'a, ADC5, Taken>,

    pub adc1_running: adc::injected::Running<'a, ADC1, Continuous, 3>, // I_U, V_U, Analog_input
    pub adc2_running: adc::injected::Running<'a, ADC2, Continuous, 3>, // Driver_temp, motor_temp, voltage_sense
    pub adc3_running: adc::injected::Running<'a, ADC3, Continuous, 2>, // I_V, V_V
    pub adc4_running: adc::injected::Running<'a, ADC4, Continuous, 1>, // V_Ref
    pub adc5_running: adc::injected::Running<'a, ADC5, Continuous, 3>, // I_W, V_W, Cpu_temp
}

#[cfg(feature = "full")]
pub type BoardCan<'a> = Can<'a>;

#[cfg(feature = "full")]
pub type BoardCrc<'a> = HardwareCrcEngine<'a>;
pub type BoardFlashBank1<'a> = Mutex<NoopRawMutex, RefCell<Bank1Region<'a, Blocking>>>;
pub type BoardFlashBank2<'a> = Mutex<NoopRawMutex, RefCell<Bank2Region<'a, Blocking>>>;

#[cfg(feature = "full")]
pub type BoardI2c<'a> = I2c<'a, Async, i2c::mode::Master>;
#[cfg(feature = "full")]
pub type BoardInverter<'a> = Inverter<'a, TIM1>;
pub struct BoardLeds<'a> {
    pub green: Output<'a>,
    pub red: Output<'a>,
}
#[cfg(feature = "full")]
pub type BoardSpi<'a> = Spi<'a, Async, spi::mode::Master>;
#[cfg(feature = "full")]
pub type BoardUart<'a> = Uart<'a, Async>;
pub type BoardUsb<'a> = usb::Driver<'a, USB>;
pub type BoardSerialNumber = [u8; 24];
