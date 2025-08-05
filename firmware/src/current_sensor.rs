use crate::config::{CurrentConfig, CurrentMeasurementConfig};
use defmt::{info, warn};
use embassy_rp::adc::{Adc, AdcPin, Async};
use embassy_rp::gpio::Pull;
use embassy_rp::{Peri, adc, bind_interrupts, dma};
use embassy_time::Timer;
use foc::Motor;
use foc::current::CurrentMeasurement;
use foc::functions::adc_conversion::{
    ConversionConstants, calculate_scaling_constants, from_adc_to_current,
};
use hardware_abstraction::current_sensor::{CurrentReader, Output};

bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => adc::InterruptHandler;
});

pub struct ThreePhaseCurrentSensor<'a, 'b, Channel>
where
    Channel: dma::Channel,
{
    adc: Adc<'a, Async>,
    dma: Peri<'a, Channel>,
    channels: [adc::Channel<'b>; 3],
    buffer: [u16; 3],
    conversion_constants: ConversionConstants,
}

impl<'a, 'p, Channel> ThreePhaseCurrentSensor<'a, 'p, Channel>
where
    Channel: dma::Channel,
{
    pub fn new(
        adc: Adc<'a, Async>,
        dma: Peri<'a, Channel>,
        pin_a: Peri<'p, impl AdcPin>,
        pin_b: Peri<'p, impl AdcPin>,
        pin_c: Peri<'p, impl AdcPin>,
        config: CurrentMeasurementConfig,
    ) -> Self {
        let conversion_constants =
            calculate_scaling_constants(config.v_ref, config.shunt_resistor, config.gain);
        Self {
            adc,
            dma,
            channels: [
                adc::Channel::new_pin(pin_a, Pull::None),
                adc::Channel::new_pin(pin_b, Pull::None),
                adc::Channel::new_pin(pin_c, Pull::None),
            ],
            buffer: [0; 3],
            conversion_constants,
        }
    }
}

impl<Channel> CurrentReader for ThreePhaseCurrentSensor<'_, '_, Channel>
where
    Channel: dma::Channel,
{
    type Error = adc::Error;
    async fn read(&mut self) -> Result<Output, Self::Error> {
        self.adc
            .read_many_multichannel(&mut self.channels, &mut self.buffer, 0, self.dma.reborrow())
            .await?;

        Ok(Output::ThreePhases(
            from_adc_to_current(self.buffer[0], &self.conversion_constants),
            from_adc_to_current(self.buffer[1], &self.conversion_constants),
            from_adc_to_current(self.buffer[2], &self.conversion_constants),
        ))
    }
}

#[embassy_executor::task]
pub async fn update_current_dma_task(
    motor: &'static Motor,
    hardware_config: Option<CurrentConfig>,
) {
    let hardware_config = match hardware_config {
        Some(c) => c,
        None => return,
    };

    let adc = Adc::new(hardware_config.adc, Irqs, adc::Config::default());
    info!("Initializing ADC...");
    let mut sensor = ThreePhaseCurrentSensor::new(
        adc,
        hardware_config.dma,
        hardware_config.phase_a,
        hardware_config.phase_b,
        hardware_config.phase_c,
        hardware_config.current_measurement_config,
    );

    loop {
        let result = update_current_dma_run_until_error(motor, &mut sensor).await;
        if let Err(e) = result {
            warn!("Error while operating ADC: {:?}", e);
        }
        info!("ADC will be restarted after 1 s.");
        Timer::after_secs(1).await;
    }
}

async fn update_current_dma_run_until_error<R: CurrentReader>(
    motor: &'static Motor,
    sensor: &mut R,
) -> Result<(), R::Error> {
    loop {
        motor.update_current(sensor).await?;
        // Run as often as possible but allow other tasks to execute too
        embassy_futures::yield_now().await;
    }
}
