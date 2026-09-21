use crate::Board;
use crate::BoardLeds;
use crate::irqs::Irqs;
use crate::serial_number::get_serial_number_as_hex;
use core::cell::RefCell;
use embassy_stm32::flash::Flash;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::usb;
use embassy_sync::blocking_mutex::Mutex;

impl Board<'static> {
    pub fn init() -> Self {
        let peripherals = crate::mcu::configure_mcu();

        let usb = usb::Driver::new(peripherals.USB, Irqs, peripherals.PA12, peripherals.PA11);

        let leds = {
            let green = Output::new(peripherals.PB9, Level::Low, Speed::Low);
            let red = Output::new(peripherals.PB7, Level::Low, Speed::Low);
            BoardLeds { green, red }
        };

        let flash = Flash::new_blocking(peripherals.FLASH).into_blocking_regions();
        let flash_bank1 = Mutex::new(RefCell::new(flash.bank1_region));
        let flash_bank2 = Mutex::new(RefCell::new(flash.bank2_region));

        let serial_number = get_serial_number_as_hex();

        Self {
            flash_bank1,
            flash_bank2,
            leds,
            usb,
            serial_number,
        }
    }
}
