use crate::app::{COMMAND_CHANNEL, EVENT_CHANNEL};
use crate::board::BoardUsb;
use communication::channel_types::EventSubscriber;
use communication::packet::{Interface, Packet};
use embassy_usb::Builder;
use embassy_usb::class::cdc_acm::{CdcAcmClass, Receiver, Sender, State};
use embassy_usb::driver::EndpointError;
use logging::info;

#[embassy_executor::task]
pub async fn task_usb(driver: BoardUsb<'static>) {
    let mut config = embassy_usb::Config::new(0x1209, 0x2aaa);
    config.manufacturer = Some("UnoProgramo");
    config.product = Some("Pyrion V1");
    let serial_hex = serial_hex(*embassy_stm32::uid::uid());
    config.serial_number = Some(core::str::from_utf8(&serial_hex).unwrap());
    config.max_power = 500;
    config.max_packet_size_0 = 64;
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut msos_descriptor = [0; 256];
    let mut control_buf = [0; 64];
    let mut state = State::default();

    let mut builder = Builder::new(
        driver,
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut msos_descriptor,
        &mut control_buf,
    );

    let class = CdcAcmClass::new(&mut builder, &mut state, 64);
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
