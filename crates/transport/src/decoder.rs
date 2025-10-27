use crate::MAX_PACKET_SIZE;
use crate::packet::Packet;
use core::fmt::Debug;
use crc_engine::CrcEngine;

#[derive(Debug)]
pub struct Decoder<T: Packet, const START_BYTE: u8> {
    position: usize,
    buffer: [u8; MAX_PACKET_SIZE],
    _phantom: core::marker::PhantomData<T>,
}

impl<T: Packet, const START_BYTE: u8> Default for Decoder<T, START_BYTE> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Packet, const START_BYTE: u8> Decoder<T, START_BYTE> {
    pub fn new() -> Self {
        Self {
            position: 0,
            buffer: [0; MAX_PACKET_SIZE],
            _phantom: core::marker::PhantomData,
        }
    }

    pub fn feed(
        &mut self,
        data: u8,
        crc: &mut impl CrcEngine,
    ) -> Option<Result<T, DecoderError<T::Error>>> {
        if self.position == 0 && data != START_BYTE {
            return None;
        }

        self.buffer[self.position] = data;
        self.position += 1;

        if self.position > 2 {
            let length = self.length();
            if self.position == length + 4 {
                let result = self.parse_command(crc);
                self.position = 0;
                return Some(result);
            }
        }

        None
    }

    fn parse_command(&self, crc: &mut impl CrcEngine) -> Result<T, DecoderError<T::Error>> {
        if !crc.check(&self.buffer[..self.length() + 4]) {
            return Err(DecoderError::InvalidCrc);
        }

        let packet = T::deserialize(&self.buffer[2..self.length() + 2])
            .map_err(DecoderError::DeserializeError)?;
        Ok(packet)
    }

    fn length(&self) -> usize {
        self.buffer[1] as usize
    }
}

/// Generic decoder error
#[derive(Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DecoderError<E> {
    InvalidCrc,
    DeserializeError(E),
}
