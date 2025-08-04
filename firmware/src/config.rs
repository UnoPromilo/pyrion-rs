use embassy_rp::peripherals::*;
use shared::units::{Resistance, Voltage};

pub struct PhysicalConfig {
    pub motor_config: MotorConfig,
    pub current: Option<CurrentConfig>,
    pub i2c: Option<I2cConfig>,
    pub angle_sensor: AngleSensorConfig,
}

pub struct MotorConfig {
    pub a_slice: PWM_SLICE0,
    pub a_high: PIN_2,
    pub a_low: PIN_3,

    pub b_slice: PWM_SLICE1,
    pub b_high: PIN_4,
    pub b_low: PIN_5,

    pub c_slice: PWM_SLICE2,
    pub c_high: PIN_6,
    pub c_low: PIN_7,
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
            motor_config: MotorConfig {
                a_slice: p.PWM_SLICE0,
                a_high: p.PIN_2,
                a_low: p.PIN_3,
                b_slice: p.PWM_SLICE1,
                b_high: p.PIN_4,
                b_low: p.PIN_5,
                c_slice: p.PWM_SLICE2,
                c_high: p.PIN_6,
                c_low: p.PIN_7,
            },
            current: Some(CurrentConfig {
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
            }),
            i2c: Some(I2cConfig {
                i2c: p.I2C0,
                scl: p.PIN_17,
                sda: p.PIN_16,
            }),
            angle_sensor: AngleSensorConfig::As5600,
        }
    }
}
