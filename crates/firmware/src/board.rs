use adc::injected::ExtTriggerSourceADC345;
use adc::trigger_edge::ExtTriggerEdge;
use adc::{Adc, Continuous, InterruptHandler, Taken};
use embassy_stm32::adc::{AdcChannel, SampleTime};
use embassy_stm32::peripherals::{ADC3, ADC4, ADC5, TIM1};
use embassy_stm32::time::khz;
use embassy_stm32::{Peripherals, bind_interrupts};
use inverter::Inverter;

bind_interrupts!(struct Irqs{
    ADC3 => InterruptHandler<ADC3>;
    ADC4 => InterruptHandler<ADC4>;
    ADC5 => InterruptHandler<ADC5>;
});

pub struct Board<'a> {
    pub adc3: Adc<'a, ADC3, Taken>,
    pub adc4: Adc<'a, ADC4, Taken>,
    pub adc5: Adc<'a, ADC5, Taken>,

    pub adc3_running: adc::injected::Running<ADC3, Continuous, 1>,
    pub adc4_running: adc::injected::Running<ADC4, Continuous, 1>,
    pub adc5_running: adc::injected::Running<ADC5, Continuous, 1>,

    pub inverter: Inverter<'a, TIM1>,
}

impl Board<'static> {
    pub fn init() -> Self {
        let peripherals = Self::configure_mcu();
        let adc_config = adc::Config::default();
        let adc3 = Adc::new(peripherals.ADC3, adc_config);
        let adc4 = Adc::new(peripherals.ADC4, adc_config);
        let adc5 = Adc::new(peripherals.ADC5, adc_config);
        let (adc3, adc3_configured) = adc3.configure_injected_ext_trigger(
            ExtTriggerSourceADC345::T3_TRGO,
            ExtTriggerEdge::Rising,
        );
        let (adc4, adc4_configured) = adc4.configure_injected_ext_trigger(
            ExtTriggerSourceADC345::T3_TRGO,
            ExtTriggerEdge::Rising,
        );

        let (adc5, adc5_configured) = adc5.configure_injected_ext_trigger(
            ExtTriggerSourceADC345::T3_TRGO,
            ExtTriggerEdge::Rising,
        );
        let adc3_running = adc3_configured.start(
            [(peripherals.PB13.degrade_adc(), SampleTime::CYCLES6_5)],
            Irqs,
        );
        let adc4_running = adc4_configured.start(
            [(peripherals.PB15.degrade_adc(), SampleTime::CYCLES6_5)],
            Irqs,
        );
        let adc5_running = adc5_configured.start(
            [(peripherals.PA8.degrade_adc(), SampleTime::CYCLES6_5)],
            Irqs,
        );

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

        Self {
            adc3,
            adc4,
            adc5,
            adc3_running,
            adc4_running,
            adc5_running,
            inverter,
        }
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
}
