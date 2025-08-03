use embassy_rp::peripherals::*;
use shared::units::{Resistance, Voltage};

pub struct PhysicalConfig {
    pub current: CurrentConfig,
    pub i2c: Option<I2cConfig>,
    pub angle_sensor: AngleSensorConfig,
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

pub struct I2cConfig {
    pub i2c: I2C0,
    pub sda: PIN_16,
    pub scl: PIN_17,
}

pub enum AngleSensorConfig {
    #[allow(unused)]
    OpenLoop,
    As5600,
}

impl PhysicalConfig {
    pub fn rp2040() -> Self {
        let p = embassy_rp::init(Default::default());
        Self {
            current: CurrentConfig {
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
            i2c: Some(I2cConfig {
                i2c: p.I2C0,
                scl: p.PIN_17,
                sda: p.PIN_16,
            }),
            angle_sensor: AngleSensorConfig::As5600,
        }
    }
}
