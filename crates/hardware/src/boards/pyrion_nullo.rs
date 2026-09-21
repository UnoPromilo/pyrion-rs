use crate::irqs::Irqs;
use crate::limits::{BoardId, BoardLimits};
use crate::serial_number::get_serial_number_as_hex;
use crate::{Board, BoardLeds};
use core::cell::RefCell;
use crc_engine::hardware::HardwareCrcEngine;
use embassy_stm32::flash::Flash;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::usb;
use embassy_sync::blocking_mutex::Mutex;
use inverter::Inverter;
use units::{ElectricCurrent, ElectricPotential, F32UnitType};
use user_config::UserConfig;

pub const BOARD_ID: BoardId = BoardId::PyrionNullo;

pub fn limits() -> BoardLimits {
    BoardLimits {
        max_bus_voltage: ElectricPotential::from_f32(24.0),
        max_phase_current: ElectricCurrent::from_f32(40.0),
    }
}

impl Board<'static> {
    pub fn init(user_config: &UserConfig) -> Self {
        let p = crate::mcu::configure_mcu();

        let crc = HardwareCrcEngine::new(p.CRC);

        let adc = super::build_adc(
            p.ADC1, p.ADC2, p.ADC3, p.ADC4, p.ADC5, p.PA1, p.PA2, p.PA3, p.PB11, p.PC3, p.PB2,
            p.PB0, p.PB1, p.PA9, p.PA8,
        );

        let inverter = Inverter::new(
            p.TIM1,
            p.PC0,
            p.PB13,
            p.PC1,
            p.PB14,
            p.PC2,
            p.PB15,
            user_config.pwm_frequency,
        );





        let usb = usb::Driver::new(p.USB, Irqs, p.PA12, p.PA11);

        let leds = {
            let green = Output::new(p.PB9, Level::Low, Speed::Low);
            let red = Output::new(p.PB7, Level::Low, Speed::Low);
            BoardLeds { green, red }
        };

        let flash = Flash::new_blocking(p.FLASH).into_blocking_regions();
        let flash_bank1 = Mutex::new(RefCell::new(flash.bank1_region));
        let flash_bank2 = Mutex::new(RefCell::new(flash.bank2_region));

        let serial_number = get_serial_number_as_hex();

        Self {
            adc,
            inverter,
            crc,
            flash_bank1,
            flash_bank2,
            leds,
            usb,
            serial_number,
        }
    }
}
