use crate::irqs::Irqs;
use crate::serial_number::get_serial_number_as_hex;
#[cfg(feature = "full")]
use crate::{Board, BoardAdc, BoardLeds};
#[cfg(not(feature = "full"))]
use crate::{Board, BoardLeds};
#[cfg(feature = "full")]
use adc::Adc;
#[cfg(feature = "full")]
use adc::injected::{ExtTriggerSourceADC12, ExtTriggerSourceADC345};
#[cfg(feature = "full")]
use adc::trigger_edge::ExtTriggerEdge;
use core::cell::RefCell;
#[cfg(feature = "full")]
use crc_engine::hardware::HardwareCrcEngine;
#[cfg(feature = "full")]
use embassy_stm32::adc::{AdcChannel, SampleTime};
#[cfg(feature = "full")]
use embassy_stm32::can::OperatingMode;
use embassy_stm32::flash::Flash;
use embassy_stm32::gpio::{Level, Output, Speed};
#[cfg(feature = "full")]
use embassy_stm32::i2c::I2c;
use embassy_stm32::pac::rcc::vals::Pllq;
use embassy_stm32::rcc::mux::{Clk48sel, Fdcansel};
#[cfg(feature = "full")]
use embassy_stm32::spi::Spi;
use embassy_stm32::time::Hertz;
#[cfg(feature = "full")]
use embassy_stm32::usart::Uart;
#[cfg(feature = "full")]
use embassy_stm32::{Peripherals, can, i2c, spi, usart, usb};
#[cfg(not(feature = "full"))]
use embassy_stm32::{Peripherals, usb};
use embassy_sync::blocking_mutex::Mutex;
#[cfg(feature = "full")]
use inverter::Inverter;
#[cfg(feature = "full")]
use user_config::UserConfig;

impl Board<'static> {
    pub fn init(#[cfg(feature = "full")] user_config: &UserConfig) -> Self {
        let peripherals = Self::configure_mcu();
        #[cfg(feature = "full")]
        let crc = HardwareCrcEngine::new(peripherals.CRC);
        #[cfg(feature = "full")]
        let onboard_i2c = {
            let mut i2c_config = i2c::Config::default();
            i2c_config.gpio_speed = Speed::VeryHigh;
            i2c_config.frequency = user_config.onboard_i2c_frequency;
            I2c::new(
                peripherals.I2C3,
                peripherals.PC8,
                peripherals.PC9,
                peripherals.DMA1_CH2,
                peripherals.DMA1_CH3,
                Irqs,
                i2c_config,
            )
        };

        #[cfg(feature = "full")]
        let ext_i2c = {
            let mut i2c_config = i2c::Config::default();
            i2c_config.gpio_speed = Speed::VeryHigh;
            i2c_config.sda_pullup = true;
            i2c_config.scl_pullup = true;
            i2c_config.frequency = user_config.external_i2c_frequency;
            I2c::new(
                peripherals.I2C4,
                peripherals.PC6,
                peripherals.PC7,
                peripherals.DMA1_CH4,
                peripherals.DMA1_CH5,
                Irqs,
                i2c_config,
            )
        };

        #[cfg(feature = "full")]
        let adc = {
            let adc_config = adc::Config::default();
            let adc1 = Adc::new(peripherals.ADC1, adc_config);
            let adc2 = Adc::new(peripherals.ADC2, adc_config);
            let adc3 = Adc::new(peripherals.ADC3, adc_config);
            let adc4 = Adc::new(peripherals.ADC4, adc_config);
            let adc5 = Adc::new(peripherals.ADC5, adc_config);

            let (adc1, adc1_configured) = adc1.configure_injected_ext_trigger(
                ExtTriggerSourceADC12::T1_TRGO,
                ExtTriggerEdge::Rising,
            );
            let (adc2, adc2_configured) = adc2.configure_injected_ext_trigger(
                ExtTriggerSourceADC12::T1_TRGO,
                ExtTriggerEdge::Rising,
            );
            let (adc3, adc3_configured) = adc3.configure_injected_ext_trigger(
                ExtTriggerSourceADC345::T1_TRGO,
                ExtTriggerEdge::Rising,
            );
            let (adc4, adc4_configured) = adc4.configure_injected_ext_trigger(
                ExtTriggerSourceADC345::T1_TRGO,
                ExtTriggerEdge::Rising,
            );

            let (adc5, adc5_configured) = adc5.configure_injected_ext_trigger(
                ExtTriggerSourceADC345::T1_TRGO,
                ExtTriggerEdge::Rising,
            );

            let adc1_running = adc1_configured.start(
                [
                    (peripherals.PA1.degrade_adc(), SampleTime::CYCLES6_5), // Current U, Ch2
                    (peripherals.PA2.degrade_adc(), SampleTime::CYCLES6_5), // Voltage U, Ch3
                    (peripherals.PA3.degrade_adc(), SampleTime::CYCLES6_5), // Analog input, Ch14
                ],
                Irqs,
            );

            let adc2_running = adc2_configured.start(
                [
                    (peripherals.PB11.degrade_adc(), SampleTime::CYCLES6_5), // Driver temp, Ch14
                    (peripherals.PC3.degrade_adc(), SampleTime::CYCLES6_5),  // Motor temp, Ch9
                    (peripherals.PB2.degrade_adc(), SampleTime::CYCLES6_5),  // Voltage sense, Ch12
                ],
                Irqs,
            );

            let adc3_running = adc3_configured.start(
                [
                    (peripherals.PB0.degrade_adc(), SampleTime::CYCLES6_5), // Current V, Ch12
                    (peripherals.PB1.degrade_adc(), SampleTime::CYCLES6_5), // Voltage V, Ch1
                ],
                Irqs,
            );

            let v_ref_int = adc4.enable_vrefint();
            let adc4_running = adc4_configured.start(
                [(v_ref_int.degrade_adc(), SampleTime::CYCLES47_5)], // V ref int, Ch18
                Irqs,
            );

            // TODO fix temperature sensor or remove it totally 
            //let temp = adc5.enable_temperature();
            let adc5_running = adc5_configured.start(
                [
                    (peripherals.PA9.degrade_adc(), SampleTime::CYCLES6_5), // Current W, Ch2
                    (peripherals.PA8.degrade_adc(), SampleTime::CYCLES6_5), // Voltage W, Ch1
                  //  (temp.degrade_adc(), SampleTime::CYCLES24_5),           // Cpu temp, Ch4
                ],
                Irqs,
            );

            BoardAdc {
                _adc1: adc1,
                _adc2: adc2,
                _adc3: adc3,
                _adc4: adc4,
                _adc5: adc5,
                adc1_running,
                adc2_running,
                adc3_running,
                adc4_running,
                adc5_running,
            }
        };

        #[cfg(feature = "full")]
        let uart = {
            let config = usart::Config::default();
            let uart = Uart::new(
                peripherals.USART1,
                peripherals.PC5,
                peripherals.PC4,
                peripherals.DMA1_CH6,
                peripherals.DMA1_CH7,
                Irqs,
                config,
            );
            match uart {
                Ok(uart) => uart,
                Err(e) => core::panic!("uart initialization error: {:?}", e),
            }
        };

        #[cfg(feature = "full")]
        let inverter = Inverter::new(
            peripherals.TIM1,
            peripherals.PC0,
            peripherals.PB13,
            peripherals.PC1,
            peripherals.PB14,
            peripherals.PC2,
            peripherals.PB15,
            user_config.pwm_frequency,
        );

        let usb = usb::Driver::new(peripherals.USB, Irqs, peripherals.PA12, peripherals.PA11);

        #[cfg(feature = "full")]
        let can = {
            let mut can = can::CanConfigurator::new(
                peripherals.FDCAN2,
                peripherals.PB5,
                peripherals.PB6,
                Irqs,
            );
            can.properties().set_extended_filter(
                can::filter::ExtendedFilterSlot::_0,
                can::filter::ExtendedFilter::accept_all_into_fifo1(),
            );
            can.set_bitrate(user_config.can_bitrate);
            can.set_fd_data_bitrate(user_config.fd_can_bitrate, false);
            can.start(OperatingMode::NormalOperationMode)
        };

        #[cfg(feature = "full")]
        let ext_spi = {
            let mut config = spi::Config::default();
            config.frequency = user_config.external_spi_frequency;
            Spi::new(
                peripherals.SPI3,
                peripherals.PC10,
                peripherals.PC12,
                peripherals.PC11,
                peripherals.DMA1_CH1,
                peripherals.DMA1_CH8,
                Irqs,
                config,
            )
        };

        #[cfg(feature = "full")]
        let onboard_spi = {
            let mut config = spi::Config::default();
            config.frequency = user_config.onboard_spi_frequency;
            Spi::new(
                peripherals.SPI1,
                peripherals.PA5,
                peripherals.PA7,
                peripherals.PA6,
                peripherals.DMA2_CH1,
                peripherals.DMA2_CH2,
                Irqs,
                config,
            )
        };

        let leds = {
            let mut green = Output::new(peripherals.PB9, Level::Low, Speed::Low);
            let mut red = Output::new(peripherals.PB7, Level::Low, Speed::Low);
            green.set_low();
            red.set_low();
            BoardLeds { green, red }
        };

        let flash = Flash::new_blocking(peripherals.FLASH).into_blocking_regions();
        let flash_bank1 = Mutex::new(RefCell::new(flash.bank1_region));
        let flash_bank2 = Mutex::new(RefCell::new(flash.bank2_region));

        let serial_number = get_serial_number_as_hex();

        #[cfg(feature = "full")]
        return Self {
            adc,
            can,
            crc,
            flash_bank1,
            flash_bank2,
            inverter,
            leds,
            ext_i2c,
            ext_spi,
            onboard_i2c,
            onboard_spi,
            uart,
            usb,
            serial_number,
        };

        #[cfg(not(feature = "full"))]
        Self {
            flash_bank1,
            flash_bank2,
            leds,
            usb,
            serial_number,
        }
    }

    fn configure_mcu() -> Peripherals {
        let config = {
            use embassy_stm32::rcc::*;
            let mut config = embassy_stm32::Config::default();
            config.rcc.hse = Some(Hse {
                freq: Hertz::mhz(24),
                mode: HseMode::Oscillator,
            });
            config.rcc.pll = Some(Pll {
                source: PllSource::HSE,
                prediv: PllPreDiv::DIV6,
                mul: PllMul::MUL85,
                divp: None,
                divq: Some(Pllq::DIV8),
                divr: Some(PllRDiv::DIV2),
            });
            config.rcc.sys = Sysclk::PLL1_R;
            config.rcc.mux.adc12sel = mux::Adcsel::SYS;
            config.rcc.mux.adc345sel = mux::Adcsel::SYS;
            config.rcc.mux.clk48sel = Clk48sel::HSI48;
            config.rcc.mux.fdcansel = Fdcansel::PLL1_Q;
            config.rcc.boost = true;
            config
        };
        embassy_stm32::init(config)
    }
}
