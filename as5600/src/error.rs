use embedded_hal_async::i2c;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug)]
pub enum Error {
    I2c(i2c::ErrorKind),
}

impl<E> From<E> for Error
where
    E: i2c::Error + Sized,
{
    fn from(value: E) -> Self {
        Error::I2c(value.kind())
    }
}
