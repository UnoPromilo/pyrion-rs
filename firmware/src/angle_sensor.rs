use crate::config::AngleSensorConfig;
use crate::i2c::I2c;
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_time::Timer;
use embedded_hal_async::i2c;
use foc::Motor;
use hardware_abstraction::angle_sensor::AngleReader;
use shared::{info, warn};
use shared::units::Angle;
use shared::units::angle::AngleAny;

type Device<I2C> = as5600::AS5600<I2C>;

pub struct AS5600Sensor<I2C> {
    device: Device<I2C>,
}

impl<I2C> From<Device<I2C>> for AS5600Sensor<I2C> {
    fn from(device: Device<I2C>) -> Self {
        Self { device }
    }
}

impl<I2C> AngleReader for AS5600Sensor<I2C>
where
    I2C: i2c::I2c,
{
    type Error = as5600::Error;

    async fn read_angle(&mut self) -> Result<AngleAny, Self::Error> {
        let value = self.device.read_angle().await?;
        Ok(AngleAny::Mechanical(Angle::from_raw(value << 4)))
    }
}

#[embassy_executor::task]
pub async fn update_angle_task(
    motor: &'static Motor,
    i2c: &'static I2c,
    config: AngleSensorConfig,
) {
    match config {
        AngleSensorConfig::OpenLoop => open_loop(motor).await,
        AngleSensorConfig::As5600 => as5600_loop(motor, i2c).await,
    }
}

async fn open_loop(_motor: &'static Motor) {
    todo!()
}

async fn as5600_loop(motor: &'static Motor, i2c: &'static I2c) {
    let i2c = if let Some(bus) = i2c {
        bus
    } else {
        panic!("I2C Bus required for AS5600 is not initialized!")
    };

    loop {
        let i2c_device = I2cDevice::new(i2c);
        let result = as5600_run_until_error(motor, i2c_device).await;
        if let Err(e) = result {
            warn!("Error while operating AS5600: {:?}", e);
        }
        info!("AS5600 will be restarted after 1 s.");
        Timer::after_secs(1).await;
    }
}

async fn as5600_run_until_error(
    motor: &'static Motor,
    i2c: impl i2c::I2c,
) -> Result<(), as5600::Error> {
    info!("Initializing AS5600 sensor...");
    let as5600 = as5600::AS5600::new(i2c, as5600::Config::default()).await?;
    let mut sensor = AS5600Sensor::from(as5600);
    loop {
        motor.update_angle(&mut sensor).await?;
    }
}
