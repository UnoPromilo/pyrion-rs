use crate::app::{COMMAND_CHANNEL, EVENT_CHANNEL};
use communication::channel_types::EventSubscriber;
use communication::packet::{Interface, Packet};
use embassy_boot_stm32::{AlignedBuffer, BlockingFirmwareState, FirmwareUpdaterConfig};
use embassy_embedded_hal::flash::partition::BlockingPartition;
use embassy_stm32::flash::{Bank1Region, Blocking, WRITE_SIZE};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_time::Duration;
use embassy_usb::Builder;
use embassy_usb::class::cdc_acm;
use embassy_usb::class::cdc_acm::{CdcAcmClass, Receiver, Sender};
use embassy_usb::class::dfu::app_mode::{DfuState, Handler, usb_dfu};
use embassy_usb::class::dfu::consts::DfuAttributes;
use embassy_usb::driver::EndpointError;
use hardware::usb::{UsbBuffers, WinUsbExt};
use hardware::{BoardFlashBank1, BoardFlashBank2, BoardUsb, configure_dfu_win_usb};
use logging::info;
use static_cell::StaticCell;

type DfuStateType<'a> =
    DfuState<DfuHandler<'a, BlockingPartition<'a, NoopRawMutex, Bank1Region<'a, Blocking>>>>;

static ALIGNED_BUFFER: StaticCell<AlignedBuffer<WRITE_SIZE>> = StaticCell::new();
static USB_BUFFERS: StaticCell<UsbBuffers> = StaticCell::new();
static CDC_STATE: StaticCell<cdc_acm::State> = StaticCell::new();
static DFU_STATE: StaticCell<DfuStateType<'static>> = StaticCell::new();

struct DfuHandler<'d, FLASH: embedded_storage::nor_flash::NorFlash> {
    firmware_state: BlockingFirmwareState<'d, FLASH>,
}

impl<FLASH: embedded_storage::nor_flash::NorFlash> Handler for DfuHandler<'_, FLASH> {
    fn enter_dfu(&mut self) {
        self.firmware_state
            .mark_dfu()
            .expect("Failed to mark DFU mode");
        cortex_m::peripheral::SCB::sys_reset();
    }
}

#[embassy_executor::task]
pub async fn task_usb(
    driver: BoardUsb<'static>,
    usb_config: embassy_usb::Config<'static>,
    flash_bank1: &'static BoardFlashBank1<'static>,
    flash_bank2: &'static BoardFlashBank2<'static>,
) {
    let usb_buffers = USB_BUFFERS.init(UsbBuffers::new());
    let cdc_state = CDC_STATE.init(cdc_acm::State::default());
    let aligned_buffer = ALIGNED_BUFFER.init(AlignedBuffer([0; WRITE_SIZE]));

    let firmware_config = FirmwareUpdaterConfig::from_linkerfile_blocking(flash_bank2, flash_bank1);
    let mut firmware_state =
        BlockingFirmwareState::from_config(firmware_config, &mut aligned_buffer.0);
    firmware_state.mark_booted().expect("Failed to mark booted");

    let dfu_handler = DfuHandler { firmware_state };
    let dfu_state = DFU_STATE.init(DfuState::new(
        dfu_handler,
        DfuAttributes::CAN_DOWNLOAD,
        Duration::from_millis(2500),
    ));

    let mut builder = Builder::new(
        driver,
        usb_config,
        &mut usb_buffers.config,
        &mut usb_buffers.bos,
        &mut usb_buffers.msos,
        &mut usb_buffers.control,
    );

    let cdc_class = CdcAcmClass::new(&mut builder, cdc_state, 64);

    builder.apply_win_usb();

    usb_dfu(&mut builder, dfu_state, |func| {
        configure_dfu_win_usb!(func);
    });
    let mut usb = builder.build();

    embassy_futures::join::join(usb.run(), run(cdc_class)).await;
}
async fn run<'a>(cdc_class: CdcAcmClass<'a, BoardUsb<'a>>) {
    let mut rx_subscriber = EVENT_CHANNEL.subscriber().expect("Can't subscribe to usb");
    let (mut tx, mut rx) = cdc_class.split();
    loop {
        tx.wait_connection().await;
        info!("USB connected");
        let _ =
            embassy_futures::join::join(handle_tx(&mut tx, &mut rx_subscriber), handle_rx(&mut rx))
                .await;
        info!("USB disconnected");
    }
}

async fn handle_tx<'a>(
    tx: &mut Sender<'a, BoardUsb<'a>>,
    rx_subscriber: &mut EventSubscriber<'a>,
) -> Result<(), EndpointError> {
    loop {
        let packet = rx_subscriber.next_message_pure().await;
        if packet.is_for_usb() {
            tx.write_packet(&packet.buffer[..packet.length]).await?;
        }
    }
}

async fn handle_rx<'a>(rx: &mut Receiver<'a, BoardUsb<'a>>) -> Result<(), EndpointError> {
    let mut buffer = [0; 64];
    loop {
        let len = rx.read_packet(&mut buffer).await?;
        let packet = Packet::from_slice(&buffer[..len], Some(Interface::Usb));
        COMMAND_CHANNEL.send(packet).await;
    }
}
