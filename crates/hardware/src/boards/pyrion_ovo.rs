use crate::irqs::Irqs;
use crate::limits::{BoardId, BoardLimits};
use crate::serial_number::get_serial_number_as_hex;
use crate::{Board, BoardLeds, SenseFilter};
use core::cell::RefCell;
use crc_engine::hardware::HardwareCrcEngine;
use drivers::{ConfigurationError, Drv8301Configuration, Drv8301Stage, SixPwmTim1};
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::flash::Flash;
use embassy_stm32::gpio::{Level, Output, Pull, Speed};
use embassy_stm32::{can, i2c, spi, usart, usb};
use embassy_sync::blocking_mutex::Mutex;
use units::{ElectricCurrent, ElectricPotential, F32UnitType};
use user_config::UserConfig;

pub const BOARD_ID: BoardId = BoardId::PyrionOvo;

pub fn validate_drv8301_configuration(
    configuration: Drv8301Configuration,
) -> Result<Drv8301Configuration, ConfigurationError> {
    configuration.validate_max_threshold_code(27)
}

pub fn limits() -> BoardLimits {
    BoardLimits {
        max_bus_voltage: ElectricPotential::from_f32(30.0),
        max_phase_current: ElectricCurrent::from_f32(60.0),
    }
}

impl Board<'static> {
    pub fn init(user_config: &UserConfig) -> Self {
        let p = crate::mcu::configure_mcu();

        let crc = HardwareCrcEngine::new(p.CRC);

        let adc = super::build_adc(
            p.ADC1, p.ADC2, p.ADC3, p.ADC4, p.ADC5, p.DMA1_CH2, p.DMA1_CH3, p.PA1, p.PA2, p.PA3,
            p.PB11, p.PC3, p.PB2, p.PB0, p.PB1, p.PA9, p.PA8,
        );

        let pwm = SixPwmTim1::new(
            p.TIM1,
            p.PC0,
            p.PB13,
            p.PC1,
            p.PB14,
            p.PC2,
            p.PB15,
            user_config.pwm_frequency,
        );

        let ext_i2c = {
            let mut config = i2c::Config::default();
            config.gpio_speed = Speed::VeryHigh;
            config.sda_pullup = true;
            config.scl_pullup = true;
            config.frequency = user_config.external_i2c_frequency;
            i2c::I2c::new(p.I2C4, p.PC6, p.PC7, p.DMA1_CH4, p.DMA1_CH5, Irqs, config)
        };

        let ext_spi = {
            let mut config = spi::Config::default();
            config.frequency = user_config.external_spi_frequency;
            spi::Spi::new(
                p.SPI3, p.PC10, p.PC12, p.PC11, p.DMA1_CH1, p.DMA1_CH8, Irqs, config,
            )
        };

        let uart = {
            let config = usart::Config::default();
            match usart::Uart::new(p.USART1, p.PC5, p.PC4, p.DMA1_CH6, p.DMA1_CH7, Irqs, config) {
                Ok(uart) => uart,
                Err(e) => core::panic!("uart initialization error: {:?}", e),
            }
        };

        let can = {
            let mut can = can::CanConfigurator::new(p.FDCAN2, p.PB5, p.PB6, Irqs);
            can.properties().set_extended_filter(
                can::filter::ExtendedFilterSlot::_0,
                can::filter::ExtendedFilter::accept_all_into_fifo1(),
            );
            can.set_bitrate(user_config.can_bitrate);
            can.set_fd_data_bitrate(user_config.fd_can_bitrate, false);
            can.start(can::OperatingMode::NormalOperationMode)
        };

        let power_stage = {
            let config = super::drv8301_spi_config();
            let bus = spi::Spi::new(
                p.SPI1, p.PA5, p.PA7, p.PA6, p.DMA2_CH1, p.DMA2_CH2, Irqs, config,
            );
            let cs = Output::new(p.PA4, Level::High, Speed::High);
            let en_gate = Output::new(p.PA0, Level::Low, Speed::Low);
            // EXTI owns PB12 in input mode; this does not configure TIM1 BKIN (AF6).
            let n_fault = ExtiInput::new(p.PB12, p.EXTI12, Pull::Up, Irqs);
            Drv8301Stage::new(pwm, bus, cs, en_gate, n_fault)
        };

        let voltage_filter = SenseFilter::new(Output::new(p.PA10, Level::Low, Speed::Low));
        let current_filter = SenseFilter::new(Output::new(p.PB10, Level::Low, Speed::Low));

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
            crc,
            can,
            ext_i2c,
            ext_spi,
            uart,
            power_stage,
            voltage_filter,
            current_filter,
            flash_bank1,
            flash_bank2,
            leds,
            usb,
            serial_number,
        }
    }
}
