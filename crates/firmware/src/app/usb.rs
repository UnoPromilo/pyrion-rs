use crate::app::{COMMAND_CHANNEL, EVENT_CHANNEL};
use crate::board::{BoardFlash, BoardUsb};
use communication::channel_types::EventSubscriber;
use communication::packet::{Interface, Packet};
use embassy_boot_stm32::{AlignedBuffer, BlockingFirmwareState, FirmwareUpdaterConfig};
use embassy_stm32::flash::WRITE_SIZE;
use embassy_time::Duration;
use embassy_usb::{msos, Builder};
use embassy_usb::class::cdc_acm::{CdcAcmClass, Receiver, Sender, State};
use embassy_usb::class::dfu::app_mode::{usb_dfu, DfuState, Handler};
use embassy_usb::class::dfu::consts::DfuAttributes;
use embassy_usb::driver::EndpointError;
use logging::info;
use user_config::UserConfig;

const DEVICE_INTERFACE_GUIDS: &[&str] = &["{EAA9A5DC-30BA-44BC-9232-606CDC875321}"];


struct DfuHandler<'d, FLASH: embedded_storage::nor_flash::NorFlash> {
    firmware_state: BlockingFirmwareState<'d, FLASH>,
}

impl<FLASH: embedded_storage::nor_flash::NorFlash> Handler for DfuHandler<'_, FLASH> {
    fn enter_dfu(&mut self) {
        self.firmware_state.mark_dfu().expect("Failed to mark DFU mode");
        cortex_m::peripheral::SCB::sys_reset();
    }
}

#[embassy_executor::task]
pub async fn task_usb(
    driver: BoardUsb<'static>,
    user_config: &'static UserConfig,
    flash: BoardFlash<'static>,
) {
    let mut config = embassy_usb::Config::new(0x1209, 0x2aaa);
    config.manufacturer = Some("UnoProgramo");
    config.product = Some(user_config.device_name);
    let serial_hex = serial_hex(embassy_stm32::uid::uid());
    config.serial_number = Some(core::str::from_utf8(&serial_hex).unwrap());
    config.max_power = 500;
    config.max_packet_size_0 = 64;
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut msos_descriptor = [0; 512];
    let mut control_buf = [0; 4096];
    let mut cdc_state = State::default();

    let mut magic = AlignedBuffer([0; WRITE_SIZE]);
    let firmware_config = FirmwareUpdaterConfig::from_linkerfile_blocking(&flash, &flash);
    let mut firmware_state = BlockingFirmwareState::from_config(firmware_config, &mut magic.0);
    firmware_state.mark_booted().expect("Failed to mark booted");
    let dfu_handler = DfuHandler { firmware_state };
    let mut dfu_state = DfuState::new(dfu_handler, DfuAttributes::CAN_DOWNLOAD, Duration::from_millis(2500));

    let mut builder = Builder::new(
        driver,
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut msos_descriptor,
        &mut control_buf,
    );

    let class = CdcAcmClass::new(&mut builder, &mut cdc_state, 64);

    builder.msos_descriptor(msos::windows_version::WIN8_1, 2);
    builder.msos_feature(msos::CompatibleIdFeatureDescriptor::new("WINUSB", ""));
    builder.msos_feature(msos::RegistryPropertyFeatureDescriptor::new(
        "DeviceInterfaceGUIDs",
        msos::PropertyData::RegMultiSz(DEVICE_INTERFACE_GUIDS),
    ));

    usb_dfu(&mut builder, &mut dfu_state, |func| {
        func.msos_feature(msos::CompatibleIdFeatureDescriptor::new("WINUSB", ""));
        func.msos_feature(msos::RegistryPropertyFeatureDescriptor::new(
            "DeviceInterfaceGUIDs",
            msos::PropertyData::RegMultiSz(DEVICE_INTERFACE_GUIDS),
        ));
    });
    let mut usb = builder.build();

    embassy_futures::join::join(usb.run(), run(class)).await;
}
async fn run<'a>(class: CdcAcmClass<'a, BoardUsb<'a>>) {
    let mut rx_subscriber = EVENT_CHANNEL.subscriber().expect("Can't subscribe to usb");
    let (mut tx, mut rx) = class.split();
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

fn serial_hex(b: [u8; 12]) -> [u8; 24] {
    fn hex(n: u8) -> u8 {
        b"0123456789abcdef"[n as usize]
    }

    let mut out = [0u8; 24];
    let mut i = 0;
    for &byte in &b {
        out[i] = hex(byte >> 4);
        out[i + 1] = hex(byte & 0x0F);
        i += 2;
    }
    out
}
