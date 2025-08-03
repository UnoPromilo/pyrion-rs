use crate::config::I2cConfig;
use embassy_rp::i2c::Async;
use embassy_rp::{bind_interrupts, i2c, peripherals};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embedded_hal::i2c::{ErrorType, Operation};
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    I2C0_IRQ => i2c::InterruptHandler<peripherals::I2C0>;
    I2C1_IRQ => i2c::InterruptHandler<peripherals::I2C1>;
});

pub type I2c = Option<Mutex<NoopRawMutex, AnyI2C>>;

static I2C_BUS: StaticCell<I2c> = StaticCell::new();

pub enum AnyI2C {
    I2C0(i2c::I2c<'static, peripherals::I2C0, Async>),
    I2C1(i2c::I2c<'static, peripherals::I2C1, Async>),
}

impl ErrorType for AnyI2C {
    type Error = i2c::Error;
}

impl embedded_hal_async::i2c::I2c<u8> for AnyI2C {
    async fn transaction(
        &mut self,
        address: u8,
        operations: &mut [Operation<'_>],
    ) -> Result<(), Self::Error> {
        match self {
            AnyI2C::I2C0(i2c) => i2c.transaction(address, operations).await,
            AnyI2C::I2C1(i2c) => i2c.transaction(address, operations).await,
        }
    }
}
impl From<i2c::I2c<'static, peripherals::I2C0, Async>> for AnyI2C {
    fn from(value: i2c::I2c<'static, peripherals::I2C0, Async>) -> Self {
        Self::I2C0(value)
    }
}

impl From<i2c::I2c<'static, peripherals::I2C1, Async>> for AnyI2C {
    fn from(value: i2c::I2c<'static, peripherals::I2C1, Async>) -> Self {
        Self::I2C1(value)
    }
}

pub fn init_i2c(physical_config: Option<I2cConfig>) -> &'static mut I2c {
    match physical_config {
        None => init_i2c_none(),
        Some(config) => init_i2c_some(config),
    }
}

fn init_i2c_some(physical_config: I2cConfig) -> &'static mut I2c {
    let mut i2c_config = i2c::Config::default();
    i2c_config.frequency = 1_000_000;
    let i2c = i2c::I2c::new_async(
        physical_config.i2c,
        physical_config.scl,
        physical_config.sda,
        Irqs,
        i2c_config,
    );

    let mutex: Mutex<NoopRawMutex, AnyI2C> = Mutex::new(i2c.into());
    I2C_BUS.init(Some(mutex))
}

fn init_i2c_none() -> &'static mut I2c {
    I2C_BUS.init(None)
}
