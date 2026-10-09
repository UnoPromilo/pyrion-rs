use embassy_stm32::gpio::Output;
use embassy_stm32::mode::Async;
use embassy_stm32::spi::{self, Spi};
use embassy_time::Timer;
use embedded_hal_async::spi::{ErrorType, Operation, SpiBus, SpiDevice};

pub(super) type DrvSpi = Spi<'static, Async, spi::mode::Master>;
pub(super) struct DrvSpiDevice<'a> {
    pub(super) bus: DrvSpi,
    pub(super) cs: Output<'a>,
}

struct CsGuard<'a, 'b>(&'b mut Output<'a>);

impl Drop for CsGuard<'_, '_> {
    fn drop(&mut self) {
        self.0.set_high();
    }
}

impl ErrorType for DrvSpiDevice<'_> {
    type Error = spi::Error;
}

impl SpiDevice<u8> for DrvSpiDevice<'_> {
    async fn transaction(
        &mut self,
        operations: &mut [Operation<'_, u8>],
    ) -> Result<(), Self::Error> {
        self.cs.set_low();
        let _cs = CsGuard(&mut self.cs);
        for operation in operations {
            match operation {
                Operation::Read(read) => self.bus.read(read).await?,
                Operation::Write(write) => self.bus.write(write).await?,
                Operation::Transfer(read, write) => self.bus.transfer(read, write).await?,
                Operation::TransferInPlace(words) => self.bus.transfer_in_place(words).await?,
                Operation::DelayNs(ns) => {
                    SpiBus::<u8>::flush(&mut self.bus).await?;
                    Timer::after_nanos(*ns as u64).await;
                }
            }
        }
        SpiBus::<u8>::flush(&mut self.bus).await
    }
}
