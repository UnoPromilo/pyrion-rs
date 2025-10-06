#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::join::join3;
use embassy_stm32::adc::{AdcChannel, AnyAdcChannel};
use embassy_stm32::gpio::OutputType;
use embassy_stm32::peripherals::{ADC1, ADC3, ADC4, ADC5, PA0, PA1, PA2};
use embassy_stm32::time::khz;
use embassy_stm32::timer::Channel;
use embassy_stm32::timer::complementary_pwm::{ComplementaryPwm, ComplementaryPwmPin};
use embassy_stm32::timer::low_level::CountingMode;
use embassy_stm32::timer::simple_pwm::PwmPin;
use embassy_stm32::{Config, Peri, Peripherals, bind_interrupts};
use embassy_time::{Duration, Instant, Timer};
use stm32_metapac::adc::vals::SampleTime;
use stm32_metapac::timer::vals::Mms;
use stm32g4_adc::injected::{ExtTriggerSourceADC12, ExtTriggerSourceADC345};
use stm32g4_adc::trigger_edge::ExtTriggerEdge;
use stm32g4_adc::{Adc, AdcInstance, InterruptHandler};
#[allow(unused_imports)]
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(
    struct Irqs {
        ADC3=> InterruptHandler<ADC3>;
        ADC4=> InterruptHandler<ADC4>;
        ADC5=> InterruptHandler<ADC5>;
    }
);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = init();
    info!("BLDC-RS!");
    let mut pwm = ComplementaryPwm::new(
        p.TIM1,
        Some(PwmPin::new(p.PA8, OutputType::PushPull)),
        Some(ComplementaryPwmPin::new(p.PB13, OutputType::PushPull)),
        None, //Some(PwmPin::new(p.PA9, OutputType::PushPull)),
        Some(ComplementaryPwmPin::new(p.PB14, OutputType::PushPull)),
        Some(PwmPin::new(p.PA10, OutputType::PushPull)),
        Some(ComplementaryPwmPin::new(p.PB15, OutputType::PushPull)),
        None,
        None,
        khz(30),
        CountingMode::CenterAlignedDownInterrupts,
    );

    pwm.set_duty(Channel::Ch1, pwm.get_max_duty() / 2);
    pwm.set_duty(Channel::Ch2, pwm.get_max_duty() / 2);
    pwm.set_duty(Channel::Ch3, pwm.get_max_duty() / 2);
    pwm.set_duty(Channel::Ch4, pwm.get_max_duty() - 1); // TODO test without -1
    pwm.set_master_output_enable(false);
    pwm.enable(Channel::Ch1);
    pwm.enable(Channel::Ch2);
    pwm.enable(Channel::Ch3);
    pwm.enable(Channel::Ch4);
    pwm.set_master_output_enable(true);
    // TODO move this to a impl
    {
        stm32_metapac::TIM1
            .cr2()
            .modify(|reg| reg.set_mms(Mms::COMPARE_OC4))
    }
    run_adc(
        p.ADC3,
        p.ADC4,
        p.ADC5,
        p.PB1.degrade_adc(),
        p.PB12.degrade_adc(),
        p.PA9.degrade_adc(),
    )
    .await;

    //read_injected_trigger_source(unsafe { Peripherals::steal() }).await;
    //read_single_conversion_source(unsafe { Peripherals::steal() }).await;
    //let mut adc = Adc::new(p.ADC2);
    //let (adc, regular) = adc.configure_regular_adc12(Default::default());
    /*let mut adc = Adc::new(p.ADC1);
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
    */
}

fn init() -> Peripherals {
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
        config.rcc.mux.adc345sel = mux::Adcsel::SYS;
        config.rcc.sys = Sysclk::PLL1_R;
    }
    embassy_stm32::init(config)
}

async fn run_adc(
    adc3: Peri<'static, ADC3>,
    adc4: Peri<'static, ADC4>,
    adc5: Peri<'static, ADC5>,
    current_a_pin: AnyAdcChannel<ADC3>,
    current_b_pin: AnyAdcChannel<ADC4>,
    current_c_pin: AnyAdcChannel<ADC5>,
) -> () {
    let config = stm32g4_adc::Config {
        ..Default::default()
    };

    let adc3 = Adc::new(adc3, config);
    let adc4 = Adc::new(adc4, config);
    let adc5 = Adc::new(adc5, config);
    let (_adc3, injected_3) = adc3
        .configure_injected_ext_trigger(ExtTriggerSourceADC345::T1_TRGO, ExtTriggerEdge::Rising);
    let (_adc4, injected_4) = adc4
        .configure_injected_ext_trigger(ExtTriggerSourceADC345::T1_TRGO, ExtTriggerEdge::Rising);
    let (_adc5, injected_5) = adc5
        .configure_injected_ext_trigger(ExtTriggerSourceADC345::T1_TRGO, ExtTriggerEdge::Rising);
    let injected_3 = injected_3.start([(current_a_pin, SampleTime::CYCLES2_5)], Irqs);
    let injected_4 = injected_4.start([(current_b_pin, SampleTime::CYCLES2_5)], Irqs);
    let injected_5 = injected_5.start([(current_c_pin, SampleTime::CYCLES2_5)], Irqs);

    loop {
        let result = join3(
            injected_3.read_next(),
            injected_4.read_next(),
            injected_5.read_next(),
        )
        .await;

        info!("{:?}", result);
        Timer::after(Duration::from_millis(100)).await;
    }
}
/*
#[allow(dead_code)]

async fn read_injected_trigger_source(p: Peripherals) {
    let adc_config = stm32g4_adc::Config {
        ..Default::default()
    };
    let adc = Adc::new(p.ADC1, adc_config);

    let (adc, injected) =
        adc.configure_injected_ext_trigger(ExtTriggerSourceADC12::T1_TRGO, ExtTriggerEdge::Rising);
    let temp_channel = adc.enable_temperature();
    let v_ref = adc.enable_vrefint();
    let injected = injected.start(
        [
            (p.PA0.degrade_adc(), SampleTime::CYCLES2_5),
            (p.PA1.degrade_adc(), SampleTime::CYCLES2_5),
            //(temp_channel.degrade_adc(), SampleTime::CYCLES2_5),
            //(v_ref.degrade_adc(), SampleTime::CYCLES2_5),
        ],
        Irqs,
    );

    let mut sum: [u128; 2] = [0; 2];
    let mut count: u128 = 0;
    let mut last_print = Instant::now();

    loop {
        count += 1;
        let values = injected.read_next().await;
        for i in 0..2 {
            sum[i] += values[i] as u128;
        }

        let elapsed = last_print.elapsed();

        //info!("elapsed: {}", elapsed);
        if elapsed > Duration::from_secs(1) {
            info!(
                "Freq: {}Hz, PA0: {}, PA1: {}",
                count / elapsed.as_millis() as u128 * 1000,
                sum[0] / count,
                sum[1] / count,
            );
            last_print = Instant::now();
            count = 0;
            sum = [0; 2];
        }
    }
}
#[allow(dead_code)]

async fn read_single_conversion_source(p: Peripherals) {
    let adc_config = stm32g4_adc::Config {
        ..Default::default()
    };
    let adc = stm32g4_adc::Adc::new(p.ADC1, adc_config);

    let (adc, injected) = adc.configure_injected_single_conversion();
    let temp = adc.enable_temperature().degrade_adc();
    let v_ref = adc.enable_vrefint().degrade_adc();
    let pa0 = p.PA0.degrade_adc();
    let pa1 = p.PA1.degrade_adc();
    let injected = injected.prepare(
        [
            (pa1, SampleTime::CYCLES6_5),
            (pa0, SampleTime::CYCLES6_5),
            (temp, SampleTime::CYCLES6_5),
            (v_ref, SampleTime::CYCLES6_5),
        ],
        Irqs,
    );

    loop {
        let values = injected.trigger_and_read().await;
        info!(
            "PA0: {}, PA1: {}, Temp: {}, V_ref:{} ",
            values[0], values[1], values[2], values[3]
        );
        Timer::after(Duration::from_millis(500)).await;
    }
}
*/
