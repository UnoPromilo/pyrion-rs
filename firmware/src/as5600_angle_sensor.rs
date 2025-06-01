use embedded_hal_async::i2c;

type Device<I2C> = as5600::AS5600<I2C>;

pub struct AS5600Sensor<I2C> {
    device: Device<I2C>,
}

impl<I2C> From<Device<I2C>> for AS5600Sensor<I2C> {
    fn from(device: Device<I2C>) -> Self {
        Self { device }
    }
}

impl<I2C> hardware_abstraction::angle_sensor::AngleSensor for AS5600Sensor<I2C>
where
    I2C: i2c::I2c,
{
    type Error = as5600::Error;

    async fn read_angle_u16(&mut self) -> Result<u16, Self::Error> {
        let value = self.device.read_angle().await?;
        Ok(value << 4)
    }
}
