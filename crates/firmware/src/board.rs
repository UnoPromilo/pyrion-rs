use adc::injected::{ExtTriggerSourceADC12, ExtTriggerSourceADC345};
use adc::trigger_edge::ExtTriggerEdge;
use adc::{Adc, Continuous, Taken};
use crc_engine::hardware::HardwareCrcEngine;
use embassy_stm32::adc::{AdcChannel, SampleTime};
use embassy_stm32::can::{Can, OperatingMode};
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::i2c::{I2c, Master};
use embassy_stm32::mode::Async;
use embassy_stm32::pac::rcc::vals::Pllq;
use embassy_stm32::peripherals::{
    ADC1, ADC2, ADC3, ADC4, ADC5, FDCAN2, I2C3, I2C4, TIM1, USART1, USB,
};
use embassy_stm32::rcc::mux::{Clk48sel, Fdcansel};
use embassy_stm32::spi::Spi;
use embassy_stm32::time::Hertz;
use embassy_stm32::usart::Uart;
use embassy_stm32::{Peripherals, bind_interrupts, can, i2c, spi};
use embassy_stm32::{usart, usb};
use inverter::Inverter;
use user_config::UserConfig;

bind_interrupts!(struct Irqs{
    ADC1_2 => adc::MultiInterruptHandler<ADC1, ADC2>;
    ADC3 => adc::SingleInterruptHandler<ADC3>;
    ADC4 => adc::SingleInterruptHandler<ADC4>;
    ADC5 => adc::SingleInterruptHandler<ADC5>;

    I2C3_EV => i2c::EventInterruptHandler<I2C3>;
    I2C3_ER => i2c::ErrorInterruptHandler<I2C3>;

    I2C4_EV => i2c::EventInterruptHandler<I2C4>;
    I2C4_ER => i2c::ErrorInterruptHandler<I2C4>;


    USART1 => usart::InterruptHandler<USART1>;

    USB_LP => usb::InterruptHandler<USB>;

    FDCAN2_IT0 => can::IT0InterruptHandler<FDCAN2>;
    FDCAN2_IT1 => can::IT1InterruptHandler<FDCAN2>;
});

pub struct Board<'a> {
    pub adc: BoardAdc<'a>,
    pub can: BoardCan<'a>,
    pub crc: BoardCrc<'a>,
    pub ext_i2c: BoardI2c<'a>,
    pub ext_spi: BoardSpi<'a>,
    pub inverter: BoardInverter<'a>,
    pub leds: BoardLeds<'a>,
    pub onboard_i2c: BoardI2c<'a>,
    pub onboard_spi: BoardSpi<'a>,
    pub uart: BoardUart<'a>,
    pub usb: BoardUsb<'a>,
    // hall
}

pub struct BoardAdc<'a> {
    pub _adc1: Adc<'a, ADC1, Taken>,
    pub _adc2: Adc<'a, ADC2, Taken>,
    pub _adc3: Adc<'a, ADC3, Taken>,
    pub _adc4: Adc<'a, ADC4, Taken>,
    pub _adc5: Adc<'a, ADC5, Taken>,

    pub adc1_running: adc::injected::Running<ADC1, Continuous, 3>, // I_U, V_U, Analog_input
    pub adc2_running: adc::injected::Running<ADC2, Continuous, 3>, // Driver_temp, motor_temp, voltage_sense
    pub adc3_running: adc::injected::Running<ADC3, Continuous, 2>, // I_V, V_V
    pub adc4_running: adc::injected::Running<ADC4, Continuous, 1>, // V_Ref
    pub adc5_running: adc::injected::Running<ADC5, Continuous, 3>, // I_W, V_W, Cpu_temp
}
pub type BoardCan<'a> = Can<'a>;
pub type BoardCrc<'a> = HardwareCrcEngine<'a>;
pub type BoardI2c<'a> = I2c<'a, Async, Master>;
pub type BoardInverter<'a> = Inverter<'a, TIM1>;
pub struct BoardLeds<'a> {
    pub green: Output<'a>,
    pub red: Output<'a>,
}
pub type BoardSpi<'a> = Spi<'a, Async>;
pub type BoardUart<'a> = Uart<'a, Async>;
pub type BoardUsb<'a> = usb::Driver<'a, USB>;

impl Board<'static> {
    pub fn init(user_config: &UserConfig) -> Result<Self, Error> {
        let peripherals = Self::configure_mcu();
        let crc = HardwareCrcEngine::new(peripherals.CRC);
        let onboard_i2c = {
            let mut i2c_config = i2c::Config::default();
            i2c_config.gpio_speed = Speed::VeryHigh;
            i2c_config.frequency = user_config.onboard_i2c_frequency;
            I2c::new(
                peripherals.I2C3,
                peripherals.PC8,
                peripherals.PC9,
                Irqs,
                peripherals.DMA1_CH2,
                peripherals.DMA1_CH3,
                i2c_config,
            )
        };

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
                Irqs,
                peripherals.DMA1_CH4,
                peripherals.DMA1_CH5,
                i2c_config,
            )
        };

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
                [(v_ref_int.degrade_adc(), SampleTime::CYCLES24_5)], // V ref int, Ch18
                Irqs,
            );

            let temp = adc5.enable_temperature();
            let adc5_running = adc5_configured.start(
                [
                    (peripherals.PA9.degrade_adc(), SampleTime::CYCLES6_5), // Current W, Ch2
                    (peripherals.PA8.degrade_adc(), SampleTime::CYCLES6_5), // Voltage W, Ch1
                    (temp.degrade_adc(), SampleTime::CYCLES47_5),           // Cpu temp, Ch4
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

        let uart = {
            let config = usart::Config::default();
            let uart = Uart::new(
                peripherals.USART1,
                peripherals.PC5,
                peripherals.PC4,
                Irqs,
                peripherals.DMA1_CH6,
                peripherals.DMA1_CH7,
                config,
            );
            match uart {
                Ok(uart) => uart,
                Err(e) => panic!("uart initialization error: {:?}", e),
            }
        };

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
                config,
            )
        };

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
                config,
            )
        };

        let leds = BoardLeds {
            green: Output::new(peripherals.PB9, Level::Low, Speed::Low),
            red: Output::new(peripherals.PB7, Level::Low, Speed::Low),
        };

        Ok(Self {
            adc,
            can,
            crc,
            inverter,
            leds,
            ext_i2c,
            ext_spi,
            onboard_i2c,
            onboard_spi,
            uart,
            usb,
        })
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

#[derive(Debug, defmt::Format)]
pub enum Error {
    AS6500(as5600::Error),
}

impl From<as5600::Error> for Error {
    fn from(e: as5600::Error) -> Self {
        Self::AS6500(e)
    }
}
