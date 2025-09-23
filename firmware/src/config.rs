use embassy_stm32::Peri;
use embassy_stm32::peripherals::{I2C1, PA10, PA14, PA15, PA8, PA9, PB10, PB11, PB13, PB14, PB15, PB7, PB8, PC0, PC1, PC2};
use shared::units::{Resistance, Voltage};

pub struct HardwareConfig {
    pub motor: MotorConfig,
    pub current: Option<CurrentConfig>,
    pub i2c: Option<I2cConfig>,
    pub angle_sensor: AngleSensorConfig,
    pub uart: Option<UartConfig>,
}

pub struct MotorConfig {
    pub a_high: Peri<'static, PA8>,
    pub b_high: Peri<'static, PA9>,
    pub c_high: Peri<'static, PA10>,

    pub a_low: Peri<'static, PB13>,
    pub b_low: Peri<'static, PB14>,
    pub c_low: Peri<'static, PB15>,
}

pub struct CurrentConfig {
    pub phase_a: Peri<'static, PC0>,
    pub phase_b: Peri<'static, PC1>,
    pub phase_c: Peri<'static, PC2>,
    pub current_measurement_config: CurrentMeasurementConfig,
}

pub struct CurrentMeasurementConfig {
    pub gain: u8,
    pub v_ref: Voltage,
    pub shunt_resistor: Resistance,
}

pub struct I2cConfig {
    pub i2c: Peri<'static, I2C1>,
    pub sda: Peri<'static, PA14>,
    pub scl: Peri<'static, PA15>,
}

pub enum AngleSensorConfig {
    #[allow(unused)]
    OpenLoop,
    As5600,
}

pub struct UartConfig {
    pub uart: Peri<'static, embassy_stm32::peripherals::USART3>,
    pub tx: Peri<'static, PB10>,
    pub rx: Peri<'static, PB11>,
}

impl HardwareConfig {
    pub fn rp2040() -> Self {
        let p = embassy_stm32::init(Default::default());
        Self {
            motor: MotorConfig {
                a_high: p.PA8,
                a_low: p.PB13,
                b_high: p.PA9,
                b_low: p.PB14,
                c_high: p.PA10,
                c_low: p.PB15,
            },
            current: Some(CurrentConfig {
                phase_a: p.PC0,
                phase_b: p.PC1,
                phase_c: p.PC2,
                current_measurement_config: CurrentMeasurementConfig {
                    gain: 20,
                    v_ref: Voltage::from_millivolts(3300),
                    shunt_resistor: Resistance::from_milliohms(100),
                },
            }),
            i2c: Some(I2cConfig {
                i2c: p.I2C1,
                sda: p.PA14,
                scl: p.PA15,
            }),
            angle_sensor: AngleSensorConfig::As5600,
            uart: Some(UartConfig {
                uart: p.USART3,
                tx: p.PB10,
                rx: p.PB11,
            }),
        }
    }
}
