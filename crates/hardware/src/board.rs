use core::cell::RefCell;
use embassy_stm32::flash::{Bank1Region, Bank2Region, Blocking};
use embassy_stm32::gpio::Output;
use embassy_stm32::peripherals::USB;
use embassy_stm32::usb;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;

#[cfg(feature = "board")]
use adc::{Adc, Continuous, Taken};
#[cfg(feature = "board")]
use crc_engine::hardware::HardwareCrcEngine;
#[cfg(feature = "board")]
use embassy_stm32::peripherals::{ADC1, ADC2, ADC3, ADC4, ADC5, TIM1};
#[cfg(feature = "board")]
use inverter::Inverter;

#[cfg(any(
    feature = "cap-external-i2c",
    feature = "cap-external-spi",
    feature = "cap-uart"
))]
use embassy_stm32::mode::Async;
#[cfg(feature = "cap-can")]
use embassy_stm32::can::Can;
#[cfg(feature = "cap-external-i2c")]
use embassy_stm32::i2c::{self, I2c};
#[cfg(feature = "cap-external-spi")]
use embassy_stm32::spi::{self, Spi};
#[cfg(feature = "cap-uart")]
use embassy_stm32::usart::Uart;

#[cfg(feature = "cap-drv8301")]
use crate::Drv8301;
#[cfg(any(feature = "cap-voltage-filter", feature = "cap-current-filter"))]
use crate::SenseFilter;

pub struct Board<'a> {
    #[cfg(feature = "board")]
    pub adc: BoardAdc<'a>,
    #[cfg(feature = "board")]
    pub inverter: BoardInverter<'a>,
    #[cfg(feature = "board")]
    pub crc: BoardCrc<'a>,
    #[cfg(feature = "cap-can")]
    pub can: BoardCan<'a>,
    #[cfg(feature = "cap-external-i2c")]
    pub ext_i2c: BoardI2c<'a>,
    #[cfg(feature = "cap-external-spi")]
    pub ext_spi: BoardSpi<'a>,
    #[cfg(feature = "cap-uart")]
    pub uart: BoardUart<'a>,
    #[cfg(feature = "cap-drv8301")]
    pub drv8301: Drv8301<'a>,
    #[cfg(feature = "cap-voltage-filter")]
    pub voltage_filter: SenseFilter<'a>,
    #[cfg(feature = "cap-current-filter")]
    pub current_filter: SenseFilter<'a>,
    pub flash_bank1: BoardFlashBank1<'a>,
    pub flash_bank2: BoardFlashBank2<'a>,
    pub leds: BoardLeds<'a>,
    pub usb: BoardUsb<'a>,
    pub serial_number: BoardSerialNumber,
}

#[cfg(feature = "board")]
pub struct BoardAdc<'a> {
    pub _adc1: Adc<'a, ADC1, Taken>,
    pub _adc2: Adc<'a, ADC2, Taken>,
    pub _adc3: Adc<'a, ADC3, Taken>,
    pub _adc4: Adc<'a, ADC4, Taken>,
    pub _adc5: Adc<'a, ADC5, Taken>,

    pub adc1_running: adc::injected::Running<'a, ADC1, Continuous, 3>,
    pub adc2_running: adc::injected::Running<'a, ADC2, Continuous, 3>,
    pub adc3_running: adc::injected::Running<'a, ADC3, Continuous, 2>,
    pub adc4_running: adc::injected::Running<'a, ADC4, Continuous, 1>,
    pub adc5_running: adc::injected::Running<'a, ADC5, Continuous, 2>,
}

#[cfg(feature = "cap-can")]
pub type BoardCan<'a> = Can<'a>;
#[cfg(feature = "board")]
pub type BoardCrc<'a> = HardwareCrcEngine<'a>;
pub type BoardFlashBank1<'a> = Mutex<NoopRawMutex, RefCell<Bank1Region<'a, Blocking>>>;
pub type BoardFlashBank2<'a> = Mutex<NoopRawMutex, RefCell<Bank2Region<'a, Blocking>>>;
#[cfg(feature = "cap-external-i2c")]
pub type BoardI2c<'a> = I2c<'a, Async, i2c::mode::Master>;
#[cfg(feature = "board")]
pub type BoardInverter<'a> = Inverter<'a, TIM1>;

pub struct BoardLeds<'a> {
    pub green: Output<'a>,
    pub red: Output<'a>,
}

#[cfg(feature = "cap-external-spi")]
pub type BoardSpi<'a> = Spi<'a, Async, spi::mode::Master>;
#[cfg(feature = "cap-uart")]
pub type BoardUart<'a> = Uart<'a, Async>;
pub type BoardUsb<'a> = usb::Driver<'a, USB>;
pub type BoardSerialNumber = [u8; 24];
