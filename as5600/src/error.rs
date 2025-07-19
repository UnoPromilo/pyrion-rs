use defmt::Format;
use embedded_hal_async::i2c;

#[derive(Debug, Format)]
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
