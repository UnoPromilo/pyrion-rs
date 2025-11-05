use crate::channel_types::{CommandChannel, EventChannel};
use crate::packet::{Interface, split_into_packets};
use command_handler::handler::execute_command;
use crc_engine::CrcEngine;
use embassy_sync::pubsub::PubSubBehavior;
use logging::error;
use transport::command::Error;
use transport::command::decoder::Decoder;
use transport::decoder::DecoderError;
use transport::event::encoder::Encoder;

pub async fn run(
    command_channel: &'static CommandChannel,
    event_channel: &'static EventChannel,
    crc: &mut impl CrcEngine,
) {
    let mut usb_decoder = Decoder::new();
    let mut serial_decoder = Decoder::new();
    let encoder = Encoder::new();
    let mut encoding_buffer = [0u8; transport::MAX_PACKET_SIZE];
    loop {
        let incoming_packet = command_channel.receive().await;
        for &byte in &incoming_packet.buffer[..incoming_packet.length] {
            let decoder = match &incoming_packet.interface {
                Interface::Serial => &mut serial_decoder,
                Interface::Usb => &mut usb_decoder,
            };

            match decoder.feed(byte, crc) {
                Some(Ok(command)) => {
                    let event = execute_command(command).await;
                    if let Some(event) = event {
                        let length = encoder.encode(&event, &mut encoding_buffer, crc);
                        for packet in split_into_packets(
                            &encoding_buffer[..length],
                            incoming_packet.interface,
                        ) {
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
}

async fn handle_error(error: DecoderError<Error>) {
    error!("Decoder error: {:?}", error);
    // TODO handle error?
}
