use embassy_stm32::adc::Adc;
use embassy_stm32::Peri;
use crate::config::{CurrentConfig, CurrentMeasurementConfig};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{Channel, Receiver, Sender};
use embassy_time::Timer;
use foc::Motor;
use foc::functions::adc_conversion::{
    ConversionConstants, calculate_scaling_constants, from_adc_to_current,
};
use hardware_abstraction::current_sensor::{CurrentReader, Output, RawOutput};
use shared::{info, warn};

pub struct ThreePhaseCurrentSensor<'a, 'b> {
    pub trigger: ThreePhaseCurrentTrigger<'a, 'b>,
    pub reader: ThreePhaseCurrentReader<'a>,
}

pub struct ThreePhaseCurrentTrigger<'a, 'b, > {
    tx: Sender<'a, CriticalSectionRawMutex, [u16; 3], 4>,
    adc: Adc<'b, ADC1>,
    channels: [adc::Channel<'b>; 3],
    buffer: [u16; 3],
}

pub struct ThreePhaseCurrentReader<'a> {
    rx: Receiver<'a, CriticalSectionRawMutex, [u16; 3], 4>,
    conversion_constants_a: ConversionConstants,
    conversion_constants_b: ConversionConstants,
    conversion_constants_c: ConversionConstants,
}

impl<'a, 'b> ThreePhaseCurrentSensor<'a, 'b> {
    pub fn new(
        adc: Adc<'b, Blocking>,
        pin_a: Peri<'b, impl AdcPin>,
        pin_b: Peri<'b, impl AdcPin>,
        pin_c: Peri<'b, impl AdcPin>,
        config: CurrentMeasurementConfig,
        channel: &'a Channel<CriticalSectionRawMutex, [u16; 3], 4>,
    ) -> Self {
        let conversion_constants =
            calculate_scaling_constants(config.v_ref, config.shunt_resistor, config.gain);

        let trigger = ThreePhaseCurrentTrigger {
            adc,
            channels: [
                adc::Channel::new_pin(pin_a, Pull::None),
                adc::Channel::new_pin(pin_b, Pull::None),
                adc::Channel::new_pin(pin_c, Pull::None),
            ],
            buffer: [0; 3],
            tx: channel.sender(),
        };

        let reader = ThreePhaseCurrentReader {
            conversion_constants_a: conversion_constants,
            conversion_constants_b: conversion_constants,
            conversion_constants_c: conversion_constants,
            rx: channel.receiver(),
        };

        Self { trigger, reader }
    }
}

impl CurrentReader for ThreePhaseCurrentReader<'_> {
    type Error = adc::Error;
    async fn wait_for_next(&mut self) -> Result<Output, Self::Error> {
        let sample = self.rx.receive().await;

        Ok(Output::ThreePhases(
            from_adc_to_current(sample[0], &self.conversion_constants_a),
            from_adc_to_current(sample[1], &self.conversion_constants_b),
            from_adc_to_current(sample[2], &self.conversion_constants_c),
        ))
    }

    async fn wait_for_next_raw(&mut self) -> Result<RawOutput, Self::Error> {
        let sample = self.rx.receive().await;

        Ok(RawOutput::ThreePhases(sample[0], sample[1], sample[2]))
    }

    async fn calibrate_current(&mut self, zero_a: u16, zero_b: u16, zero_c: u16) {
        self.conversion_constants_a.recalculate_mid_value(zero_a);
        self.conversion_constants_b.recalculate_mid_value(zero_b);
        self.conversion_constants_c.recalculate_mid_value(zero_c);
    }
}

impl ThreePhaseCurrentTrigger<'_, '_> {
    pub fn update_buffer(&mut self) {
        for (ch, dst) in self.channels.iter_mut().zip(self.buffer.iter_mut()) {
            *dst = self.adc.blocking_read(ch).unwrap();
        }
        self.tx.try_send(self.buffer).ok();
    }
}

pub fn setup_current_sensor(
    hardware_config: Option<CurrentConfig>,
    channel: &'static Channel<CriticalSectionRawMutex, [u16; 3], 4>,
) -> Option<ThreePhaseCurrentSensor<'static, 'static>> {
    let hardware_config = match hardware_config {
        Some(c) => c,
        None => return None,
    };

    let adc = Adc::new_blocking(hardware_config.adc, adc::Config::default());
    Some(ThreePhaseCurrentSensor::new(
        adc,
        hardware_config.phase_a,
        hardware_config.phase_b,
        hardware_config.phase_c,
        hardware_config.current_measurement_config,
        channel,
    ))
}

#[embassy_executor::task]
pub async fn update_current_task(motor: &'static Motor, mut current_reader: ThreePhaseCurrentReader<'static>) {
    info!("Initializing ADC...");
    loop {
        let result = motor.update_current_task(&mut current_reader).await;
        if let Err(e) = result {
            warn!("Error while operating ADC: {:?}", e);
        }
        info!("ADC will be restarted after 1 s.");
        Timer::after_secs(1).await;
    }
}
