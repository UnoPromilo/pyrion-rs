use crate::packet::Packet;
use crc_engine::CrcEngine;

#[derive(Debug)]
pub struct Encoder<T: Packet, const START_BYTE: u8> {
    _phantom: core::marker::PhantomData<T>,
}

impl<T: Packet, const START_BYTE: u8> Default for Encoder<T, START_BYTE> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Packet, const START_BYTE: u8> Encoder<T, START_BYTE> {
    pub fn new() -> Self {
        Self {
            _phantom: core::marker::PhantomData,
        }
    }

    pub fn encode(&self, packet: &T, buffer: &mut [u8], crc: &mut impl CrcEngine) -> usize {
        buffer[0] = START_BYTE;
        let payload_len = packet.serialize(&mut buffer[2..]);
        buffer[1] = payload_len as u8;
        let crc_val = crc.calculate(&buffer[..2 + payload_len]);
        let crc_bytes = crc_val.to_le_bytes();
        buffer[2 + payload_len] = crc_bytes[0];
        buffer[2 + payload_len + 1] = crc_bytes[1];
        2 + payload_len + 2
    }
}

// TODO: Add tests, especially for the encode -> decode cycle