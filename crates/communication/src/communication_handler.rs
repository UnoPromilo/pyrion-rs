use crate::channel_types::{CommandChannel, EventChannel};
use crate::packet::{Interface, Packet, split_into_packets};
use command_handler::handler::execute_command;
use command_handler::telemetry::get_telemetry;
use crc_engine::CrcEngine;
use embassy_futures::select::{Either, select};
use embassy_sync::pubsub::PubSubBehavior;
use embassy_time::Duration;
use logging::{error, warn};
use transport::Event;
use transport::command::Error;
use transport::command::decoder::Decoder;
use transport::decoder::DecoderError;
use transport::event::encoder::Encoder;

pub async fn run(
    command_channel: &'static CommandChannel,
    event_channel: &'static EventChannel,
    crc: &mut impl CrcEngine,
) {
    let mut telemetry_ticker = embassy_time::Ticker::every(Duration::from_hz(10));
    let mut usb_decoder = Decoder::new();
    let mut serial_decoder = Decoder::new();
    let encoder = Encoder::new();
    let mut encoding_buffer = [0u8; transport::MAX_PACKET_SIZE];
    let receiver = command_channel.receiver();
    loop {
        match select(telemetry_ticker.next(), receiver.receive()).await {
            Either::First(_) => {
                broadcast_telemetry(&encoder, &mut encoding_buffer, crc, event_channel).await;
            }
            Either::Second(incoming_packet) => {
                handle_incoming_packet(
                    event_channel,
                    crc,
                    &mut usb_decoder,
                    &mut serial_decoder,
                    &encoder,
                    &mut encoding_buffer,
                    &incoming_packet,
                )
                .await;
            }
        }
    }
}

async fn handle_incoming_packet(
    event_channel: &EventChannel,
    crc: &mut impl CrcEngine,
    usb_decoder: &mut Decoder,
    serial_decoder: &mut Decoder,
    encoder: &Encoder,
    encoding_buffer: &mut [u8],
    incoming_packet: &Packet,
) {
    let decoder = match &incoming_packet.interface {
        Some(Interface::Serial) => serial_decoder,
        Some(Interface::Usb) => usb_decoder,
        None => {
            warn!(
                "Received broadcast packet without interface specified - broadcasts are output-only"
            );
            return;
        }
    };
    for &byte in &incoming_packet.buffer[..incoming_packet.length] {
        match decoder.feed(byte, crc) {
            Some(Ok(command)) => {
                let event = execute_command(command).await;
                if let Some(event) = event {
                    let length = encoder.encode(&event, encoding_buffer, crc);
                    for packet in
                        split_into_packets(&encoding_buffer[..length], incoming_packet.interface)
                    {
                        event_channel.publish_immediate(packet);
                    }
                }
            }
            Some(Err(error)) => {
                handle_error(error).await;
            }
            None => {} // Still parsing
        }
    }
}

async fn broadcast_telemetry(
    encoder: &Encoder,
    encoding_buffer: &mut [u8],
    crc: &mut impl CrcEngine,
    event_channel: &EventChannel,
) {
    let telemetry = get_telemetry();
    let length = encoder.encode(&Event::Telemetry(telemetry), encoding_buffer, crc);
    for packet in split_into_packets(&encoding_buffer[..length], None) {
        event_channel.publish_immediate(packet);
    }
}

async fn handle_error(error: DecoderError<Error>) {
    error!("Decoder error: {:?}", error);
    // TODO handle error?
}
