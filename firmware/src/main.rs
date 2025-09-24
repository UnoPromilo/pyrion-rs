#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::adc::{Adc, SampleTime};
use embassy_time::{Duration, Timer};
#[allow(unused_imports)]
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
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
        config.rcc.sys = Sysclk::PLL1_R;
    }
    let p = embassy_stm32::init(config);
    info!("Hello World!");

    let mut adc = Adc::new(p.ADC1);
    adc.set_sample_time(SampleTime::CYCLES24_5);
    let mut pin = p.PA0; // Arduino pin A0;

    let mut temperature = adc.enable_temperature();
    let mut vrefint = adc.enable_vrefint();
    Timer::after(Duration::from_millis(100)).await;

    let v_ref_int_sample = adc.blocking_read(&mut vrefint);
    let convert_to_millivolts = |sample| {
        const V_REF_INT_MV: u32 = 1210; // mV

        (u32::from(sample) * V_REF_INT_MV / u32::from(v_ref_int_sample)) as u16
    };

    let convert_to_celsius = |sample| {
        const V25: i32 = 760; // mV
        const AVG_SLOPE: f32 = 2.5; // mV/C

        let sample_mv = convert_to_millivolts(sample) as i32;
        (sample_mv - V25) as f32 / AVG_SLOPE + 25.0
    };

    info!("VrefInt: {}", v_ref_int_sample);
    const MAX_ADC_SAMPLE: u16 = (1 << 12) - 1;
    info!("VCCA: {} mV", convert_to_millivolts(MAX_ADC_SAMPLE));

    loop {
        let pin_v = adc.blocking_read(&mut pin);
        info!("PA0: {} ({} mV)", pin_v, convert_to_millivolts(pin_v));

        let temp_v = adc.blocking_read(&mut temperature);
        let celsius = convert_to_celsius(temp_v);
        info!("Internal temp: {} ({} C)", temp_v, celsius);

        let v_ref = adc.blocking_read(&mut vrefint);
        info!("V ref int: {}", v_ref);

        Timer::after_millis(100).await;
    }
}
