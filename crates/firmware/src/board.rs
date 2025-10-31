use adc::injected::ExtTriggerSourceADC345;
use adc::trigger_edge::ExtTriggerEdge;
use adc::{Adc, Continuous, Taken};
use as5600::AS5600;
use crc_engine::hardware::HardwareCrcEngine;
use embassy_stm32::adc::{AdcChannel, SampleTime};
use embassy_stm32::gpio::Speed;
use embassy_stm32::i2c::{I2c, Master};
use embassy_stm32::mode::Async;
use embassy_stm32::peripherals::{ADC3, ADC4, ADC5, I2C1, LPUART1, TIM1};
use embassy_stm32::time::khz;
use embassy_stm32::usart;
use embassy_stm32::usart::Uart;
use embassy_stm32::{bind_interrupts, i2c, Peripherals};
use embassy_time::{Duration, Timer};
use inverter::Inverter;

bind_interrupts!(struct Irqs{
    ADC3 => adc::InterruptHandler<ADC3>;
    ADC4 => adc::InterruptHandler<ADC4>;
    ADC5 => adc::InterruptHandler<ADC5>;

    I2C1_EV => i2c::EventInterruptHandler<I2C1>;
    I2C1_ER => i2c::ErrorInterruptHandler<I2C1>;

    LPUART1 => usart::InterruptHandler<LPUART1>;

});

pub struct Board<'a> {
    pub adc: BoardAdc<'a>,
    pub inverter: BoardInverter<'a>,
    pub encoder: BoardEncoder<'a>,
    pub uart: BoardUart<'a>,
    pub crc: BoardCrc<'a>,
}

pub struct BoardAdc<'a> {
    pub _adc3: Adc<'a, ADC3, Taken>,
    pub _adc4: Adc<'a, ADC4, Taken>,
    pub _adc5: Adc<'a, ADC5, Taken>,

    pub adc3_running: adc::injected::Running<ADC3, Continuous, 1>, // I_U
    pub adc4_running: adc::injected::Running<ADC4, Continuous, 3>, // I_V, V_REF, V_BUS
    pub adc5_running: adc::injected::Running<ADC5, Continuous, 2>, // I_W, Temp
}

pub type BoardInverter<'a> = Inverter<'a, TIM1>;
pub type BoardEncoder<'a> = AS5600<I2c<'a, Async, Master>>;
pub type BoardUart<'a> = Uart<'a, Async>;
pub type BoardCrc<'a> = HardwareCrcEngine<'a>;

impl Board<'static> {
    pub async fn init() -> Result<Self, Error> {
        let peripherals = Self::configure_mcu();
        let crc = HardwareCrcEngine::new(peripherals.CRC);
        let as5600 = {
            let mut i2c_config = i2c::Config::default();
            i2c_config.gpio_speed = Speed::VeryHigh;
            i2c_config.frequency = khz(100);
            let i2c = I2c::new(
                peripherals.I2C1,
                peripherals.PB8,
                peripherals.PB9,
                Irqs,
                peripherals.DMA1_CH2,
                peripherals.DMA1_CH3,
                i2c_config,
            );

            AS5600::new(i2c, as5600::Config::default()).await?
        };

        let adc = {
            let adc_config = adc::Config::default();
            let adc3 = Adc::new(peripherals.ADC3, adc_config);
            let adc4 = Adc::new(peripherals.ADC4, adc_config);
            let adc5 = Adc::new(peripherals.ADC5, adc_config);

            let v_ref_int = adc4.enable_vrefint();
            let temp = adc5.enable_temperature();

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
            let adc3_running = adc3_configured.start(
                [(peripherals.PB13.degrade_adc(), SampleTime::CYCLES6_5)],
                Irqs,
            );
            let adc4_running = adc4_configured.start(
                [
                    (peripherals.PB15.degrade_adc(), SampleTime::CYCLES6_5),
                    (v_ref_int.degrade_adc(), SampleTime::CYCLES24_5),
                    (peripherals.PB14.degrade_adc(), SampleTime::CYCLES6_5),
                ],
                Irqs,
            );
            let adc5_running = adc5_configured.start(
                [
                    (peripherals.PA8.degrade_adc(), SampleTime::CYCLES6_5),
                    (temp.degrade_adc(), SampleTime::CYCLES24_5),
                ],
                Irqs,
            );

            // Wait for ADC startup
            Timer::after(Duration::from_millis(100)).await;

            BoardAdc {
                _adc3: adc3,
                _adc4: adc4,
                _adc5: adc5,
                adc3_running,
                adc4_running,
                adc5_running,
            }
        };

        let uart = {
            let config = usart::Config::default();
            let uart = Uart::new(
                peripherals.LPUART1,
                peripherals.PA3,
                peripherals.PA2,
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
            peripherals.PC13,
            peripherals.PC1,
            peripherals.PB0,
            peripherals.PC3,
            peripherals.PC5,
            khz(30),
        );

        Ok(Self {
            adc,
            inverter,
            encoder: as5600,
            uart,
            crc,
        })
    }

    fn configure_mcu() -> Peripherals {
        let config = {
            use embassy_stm32::rcc::*;
            let mut config = embassy_stm32::Config::default();
            config.rcc.pll = Some(Pll {
                source: PllSource::HSI,
                prediv: PllPreDiv::DIV4,
                mul: PllMul::MUL85,
                divp: None,
                divq: None,
                // Main system clock at 170 MHz
                divr: Some(PllRDiv::DIV2),
            });
            config.rcc.mux.adc12sel = mux::Adcsel::SYS;
            config.rcc.mux.adc345sel = mux::Adcsel::SYS;
            config.rcc.sys = Sysclk::PLL1_R;
            config
        };
        embassy_stm32::init(config)
    }

    pub fn split(
        self,
    ) -> (
        BoardAdc<'static>,
        BoardInverter<'static>,
        BoardEncoder<'static>,
        BoardUart<'static>,
        BoardCrc<'static>,
    ) {
        (self.adc, self.inverter, self.encoder, self.uart, self.crc)
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
