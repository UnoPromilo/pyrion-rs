use crate::app::communication::CONTROL_COMMAND_CHANNEL;
use crate::app::safety;
use crate::app::{COMMAND_CHANNEL, EVENT_CHANNEL};
use communication::channel_types::EventSubscriber;
use communication::packet::{Interface, Packet};
use controller_shared::command::ControlCommand;
use embassy_boot_stm32::{AlignedBuffer, BlockingFirmwareState, FirmwareUpdaterConfig};
use embassy_embedded_hal::flash::partition::BlockingPartition;
use embassy_futures::join::join3;
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
use logging::{error, info};
use static_cell::StaticCell;

type FirmwareStateType<'a> =
    BlockingFirmwareState<'a, BlockingPartition<'a, NoopRawMutex, Bank1Region<'a, Blocking>>>;
type DfuStateType = DfuState<DfuHandler>;

static ALIGNED_BUFFER: StaticCell<AlignedBuffer<WRITE_SIZE>> = StaticCell::new();
static USB_BUFFERS: StaticCell<UsbBuffers> = StaticCell::new();
static CDC_STATE: StaticCell<cdc_acm::State> = StaticCell::new();
static DFU_STATE: StaticCell<DfuStateType> = StaticCell::new();

struct DfuHandler;

impl Handler for DfuHandler {
    fn enter_dfu(&mut self) {
        safety::request_dfu_shutdown();
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

    let dfu_handler = DfuHandler;
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

    join3(
        usb.run(),
        run(cdc_class),
        enter_dfu_when_safe(firmware_state),
    )
    .await;
}

async fn enter_dfu_when_safe(mut firmware_state: FirmwareStateType<'_>) {
    loop {
        let request_id = safety::wait_for_dfu_shutdown_request().await;
        let shutdown = async {
            CONTROL_COMMAND_CHANNEL
                .send(ControlCommand::InhibitForDfu { request_id })
                .await;
            safety::wait_for_dfu_output_inhibited(request_id).await;
        };
        if embassy_time::with_timeout(Duration::from_millis(100), shutdown)
            .await
            .is_err()
        {
            error!("DFU shutdown acknowledgement timed out");
            continue;
        }
        firmware_state.mark_dfu().expect("Failed to mark DFU mode");
        cortex_m::peripheral::SCB::sys_reset();
    }
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
