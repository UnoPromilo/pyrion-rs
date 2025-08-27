use embassy_rp::Peri;
use embassy_rp::peripherals::*;
use shared::units::{Resistance, Voltage};

pub struct HardwareConfig {
    pub motor: MotorConfig,
    pub current: Option<CurrentConfig>,
    pub i2c: Option<I2cConfig>,
    pub angle_sensor: AngleSensorConfig,
    pub uart: Option<UartConfig>,
    pub core1: Peri<'static, CORE1>,
}

pub struct MotorConfig {
    pub a_slice: Peri<'static, PWM_SLICE1>,
    pub a_high: Peri<'static, PIN_2>,
    pub a_low: Peri<'static, PIN_3>,

    pub b_slice: Peri<'static, PWM_SLICE2>,
    pub b_high: Peri<'static, PIN_4>,
    pub b_low: Peri<'static, PIN_5>,

    pub c_slice: Peri<'static, PWM_SLICE3>,
    pub c_high: Peri<'static, PIN_6>,
    pub c_low: Peri<'static, PIN_7>,
}

pub struct CurrentConfig {
    pub adc: Peri<'static, ADC>,
    pub phase_a: Peri<'static, PIN_26>,
    pub phase_b: Peri<'static, PIN_27>,
    pub phase_c: Peri<'static, PIN_28>,
    pub current_measurement_config: CurrentMeasurementConfig,
}

pub struct CurrentMeasurementConfig {
    pub gain: u8,
    pub v_ref: Voltage,
    pub shunt_resistor: Resistance,
}

pub struct I2cConfig {
    pub i2c: Peri<'static, I2C0>,
    pub sda: Peri<'static, PIN_16>,
    pub scl: Peri<'static, PIN_17>,
}

pub enum AngleSensorConfig {
    #[allow(unused)]
    OpenLoop,
    As5600,
}

pub struct UartConfig {
    pub uart: Peri<'static, UART0>,
    pub tx: Peri<'static, PIN_0>,
    pub rx: Peri<'static, PIN_1>,
    pub tx_dma: Peri<'static, DMA_CH0>,
    pub rx_dma: Peri<'static, DMA_CH1>,
}

impl HardwareConfig {
    pub fn rp2040() -> Self {
        let p = embassy_rp::init(Default::default());
        Self {
            core1: p.CORE1,
            motor: MotorConfig {
                a_slice: p.PWM_SLICE1,
                a_high: p.PIN_2,
                a_low: p.PIN_3,
                b_slice: p.PWM_SLICE2,
                b_high: p.PIN_4,
                b_low: p.PIN_5,
                c_slice: p.PWM_SLICE3,
                c_high: p.PIN_6,
                c_low: p.PIN_7,
            },
            current: Some(CurrentConfig {
                adc: p.ADC,
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
            uart: Some(UartConfig {
                uart: p.UART0,
                tx: p.PIN_0,
                rx: p.PIN_1,
                tx_dma: p.DMA_CH0,
                rx_dma: p.DMA_CH1,
            }),
        }
    }
}
