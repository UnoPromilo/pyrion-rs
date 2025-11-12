#![no_std]

use embassy_boot::FirmwareUpdaterError;
use embassy_boot_stm32::{AlignedBuffer, FirmwareUpdater, FirmwareUpdaterConfig};
use embassy_embedded_hal::adapter::BlockingAsync;
use embassy_embedded_hal::flash::partition::Partition;
use embassy_stm32::flash::{Blocking, Flash, WRITE_SIZE};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;

pub type FwBuffer = AlignedBuffer<WRITE_SIZE>;
pub type FwUpdater<'d> = FirmwareUpdater<
    'd,
    Partition<'d, NoopRawMutex, BlockingAsync<Flash<'d, Blocking>>>,
    Partition<'d, NoopRawMutex, BlockingAsync<Flash<'d, Blocking>>>,
>;

pub struct FirmwareUpdateManager<'d> {
    updater: FwUpdater<'d>,
}

impl<'d> FirmwareUpdateManager<'d> {
    pub fn new(
        flash: &'d Mutex<NoopRawMutex, BlockingAsync<Flash<'d, Blocking>>>,
        fw_buffer: &'d mut FwBuffer,
    ) -> Self {
        let config = FirmwareUpdaterConfig::from_linkerfile(flash, flash);
        let updater = FirmwareUpdater::new(config, &mut fw_buffer.0);
        Self { updater }
    }

    pub async fn write_block(
        &mut self,
        data: &[u8],
        offset: usize,
    ) -> Result<(), FirmwareUpdaterError> {
        self.updater.write_firmware(offset, data).await
    }

    pub async fn finish_update(&mut self) -> Result<(), FirmwareUpdaterError> {
        self.updater.mark_updated().await?;

        cortex_m::peripheral::SCB::sys_reset();
    }
}

pub fn new_fw_buffer() -> FwBuffer {
    AlignedBuffer([0; WRITE_SIZE])
}
