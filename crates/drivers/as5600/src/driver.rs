use crate::config::Config;
use crate::error::Error;
use crate::registers::*;
use embedded_hal_async::i2c;
use logging::debug;

pub struct AS5600<I2C> {
    i2c: I2C,
    config: Config,
}

type Result<T, E = Error> = core::result::Result<T, E>;

impl<I2C> AS5600<I2C>
where
    I2C: i2c::I2c,
{
    pub async fn new(i2c: I2C, config: Config) -> Result<Self> {
        debug!("Initializing AS5600");
        let mut as5600 = Self { i2c, config };
        as5600.write_config().await?;
        Ok(as5600)
    }

    pub async fn write_config(&mut self) -> Result<()> {
        self.write_u8(Register::ConfHigh, self.config.get_high_config_byte())
            .await?;
        self.write_u8(Register::ConfLow, self.config.get_low_config_byte())
            .await?;

        self.write_u8(Register::ZPosHigh, self.config.get_high_z_pos())
            .await?;
        self.write_u8(Register::ZPosLow, self.config.get_low_z_pos())
            .await?;

        self.write_u8(Register::MPosHigh, self.config.get_high_m_pos())
            .await?;
        self.write_u8(Register::MPosLow, self.config.get_low_m_pos())
            .await?;

        Ok(())
    }

    pub async fn read_raw_angle(&mut self) -> Result<u16> {
        self.read_u16(Register::RawAngle).await
    }

    /// 0-4095
    pub async fn read_angle(&mut self) -> Result<u16> {
        self.read_u16(Register::Angle).await
    }

    pub async fn read_agc(&mut self) -> Result<u8> {
        self.read_u8(Register::Agc).await
    }

    pub async fn read_magnitude(&mut self) -> Result<u16> {
        self.read_u16(Register::Magnitude).await
    }

    async fn read_u8(&mut self, address: Register) -> Result<u8> {
        let mut buffer = [0u8; 1];
        self.i2c
            .write_read(ADDRESS, &[address.into()], &mut buffer)
            .await?;

        Ok(buffer[0])
    }

    async fn read_u16(&mut self, address: Register) -> Result<u16> {
        let mut buffer = [0u8; 2];
        self.i2c
            .write_read(ADDRESS, &[address.into()], &mut buffer)
            .await?;

        Ok(u16::from_le_bytes([buffer[1], buffer[0]]))
    }

    async fn write_u8(&mut self, address: Register, value: u8) -> Result<()> {
        self.i2c.write(ADDRESS, &[address.into(), value]).await?;
        Ok(())
    }
}
