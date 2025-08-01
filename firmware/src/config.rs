use embassy_rp::peripherals::*;
use shared::units::{Resistance, Voltage};

pub struct PhysicalConfig {
    pub current_config: CurrentConfig,
}

pub struct CurrentConfig {
    pub adc: ADC,
    pub dma: DMA_CH0,
    pub phase_a: PIN_26,
    pub phase_b: PIN_27,
    pub phase_c: PIN_28,
    pub current_measurement_config: CurrentMeasurementConfig,
}

pub struct CurrentMeasurementConfig {
    pub gain: u8,
    pub v_ref: Voltage,
    pub shunt_resistor: Resistance,
}

impl PhysicalConfig {
    pub fn rp2040() -> Self {
        let p = embassy_rp::init(Default::default());
        Self {
            current_config: CurrentConfig {
                adc: p.ADC,
                dma: p.DMA_CH0,
                phase_a: p.PIN_26,
                phase_b: p.PIN_27,
                phase_c: p.PIN_28,
                current_measurement_config: CurrentMeasurementConfig {
                    gain: 20,
                    v_ref: Voltage::from_millivolts(3300),
                    shunt_resistor: Resistance::from_milliohms(100),
                },
            },
        }
    }
}
