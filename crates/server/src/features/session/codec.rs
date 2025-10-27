use crate::features::session::error::{DecoderError, EncoderError};
use crc_engine::software::SoftwareCrcEngine;
use prost::bytes::BytesMut;
use tokio_util::bytes::Buf;
use tokio_util::codec::{Decoder, Encoder};

#[derive(Debug)]
pub struct DeviceCoded {
    decoder: transport::event::decoder::Decoder,
    encoder: transport::command::encoder::Encoder,
    crc_engine: SoftwareCrcEngine,
}

impl Default for DeviceCoded {
    fn default() -> Self {
        Self::new()
    }
}

impl DeviceCoded {
    pub fn new() -> Self {
        Self {
            decoder: transport::event::decoder::Decoder::new(),
            encoder: transport::command::encoder::Encoder::new(),
            crc_engine: SoftwareCrcEngine::new(),
        }
    }
}

impl Decoder for DeviceCoded {
    type Item = transport::Event;
    type Error = DecoderError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        while !src.is_empty() {
            let byte = src[0];
            src.advance(1);
            match self.decoder.feed(byte, &mut self.crc_engine) {
                Some(Ok(event)) => {
                    return Ok(Some(event));
                }
                Some(Err(error)) => {
                    return Err(error.into());
                }
                None => {}
            }
        }

        Ok(None)
    }
}

impl Encoder<transport::Command> for DeviceCoded {
    type Error = EncoderError;

    fn encode(&mut self, item: transport::Command, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let mut buffer = [0; transport::MAX_PACKET_SIZE];
        let len = self
            .encoder
            .encode(&item, &mut buffer, &mut self.crc_engine);

        dst.reserve(len);
        dst.extend_from_slice(&buffer[..len]);
        Ok(())
    }
}
