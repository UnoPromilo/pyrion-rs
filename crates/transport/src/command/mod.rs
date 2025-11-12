use crate::helpers::decode_u32;
use crate::packet::Packet;
use core::array::TryFromSliceError;

pub mod decoder;
pub mod encoder;

pub const FIRMWARE_BLOCK_MAX_DATA_SIZE: usize = u8::MAX as usize - (size_of::<u32>() * 2) - 1;

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[allow(clippy::large_enum_variant)]
pub enum Command {
    IntroduceYourself,                 // 0x01
    Stop,                              // 0x02
    WriteFirmwareBlock(FirmwareBlock), // 0x10
    FinalizeFirmwareUpdate,            //0x11
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FirmwareBlock {
    pub offset: u32,
    pub length: u32,
    pub data: [u8; FIRMWARE_BLOCK_MAX_DATA_SIZE],
}

impl Packet for Command {
    type Error = Error;

    fn deserialize(data: &[u8]) -> Result<Self, Self::Error> {
        let cmd_byte = data[0];
        match cmd_byte {
            0x01 => Ok(Command::IntroduceYourself),
            0x02 => Ok(Command::Stop),
            0x10 => {
                let packet = FirmwareBlock::deserialize(&data[1..])?;
                Ok(Command::WriteFirmwareBlock(packet))
            }
            0x11 => Ok(Command::FinalizeFirmwareUpdate),
            _ => Err(Error::CommandNotFound),
        }
    }

    fn serialize(&self, buffer: &mut [u8]) -> usize {
        match self {
            Command::IntroduceYourself => {
                buffer[0] = 0x01;
                1
            }
            Command::Stop => {
                buffer[0] = 0x02;
                1
            }
            Command::WriteFirmwareBlock(packet) => {
                buffer[0] = 0x10;
                packet.serialize(&mut buffer[1..]) + 1
            }
            Command::FinalizeFirmwareUpdate => {
                buffer[0] = 0x11;
                1
            }
        }
    }
}

impl FirmwareBlock {
    pub fn serialize(&self, buffer: &mut [u8]) -> usize {
        buffer[..4].copy_from_slice(&self.offset.to_le_bytes());
        buffer[4..8].copy_from_slice(&self.length.to_le_bytes());
        buffer[8..self.length as usize + 8].copy_from_slice(&self.data[..self.length as usize]);
        FIRMWARE_BLOCK_MAX_DATA_SIZE + 8
    }

    pub fn deserialize(data: &[u8]) -> Result<Self, Error> {
        let offset = decode_u32(&data[..4])?;
        let length = decode_u32(&data[4..8])?;
        let mut buffer: [u8; FIRMWARE_BLOCK_MAX_DATA_SIZE] = [0; FIRMWARE_BLOCK_MAX_DATA_SIZE];
        buffer[..length as usize].copy_from_slice(&data[8..length as usize + 8]);
        Ok(Self {
            offset,
            length,
            data: buffer,
        })
    }

    pub fn slice(&self) -> &[u8] {
        &self.data[..self.length as usize]
    }
}

#[derive(Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    CommandNotFound,
    InvalidContent,
}

impl From<TryFromSliceError> for Error {
    fn from(_: TryFromSliceError) -> Self {
        Error::InvalidContent
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MAX_PACKET_SIZE;

    #[test]
    fn unknown_command_should_return_error() {
        let buffer = [0x00];
        let result = Command::deserialize(&buffer);
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), Error::CommandNotFound);
    }

    #[test]
    fn introduce_yourself_command() {
        let mut buffer = [0; 100];
        let len = Command::IntroduceYourself.serialize(&mut buffer);
        let result = Command::deserialize(&buffer[..len]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Command::IntroduceYourself);
    }

    #[test]
    fn stop_command() {
        let mut buffer = [0; 100];
        let len = Command::Stop.serialize(&mut buffer);
        let result = Command::deserialize(&buffer[..len]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Command::Stop);
    }

    #[test]
    fn write_firmware_block_command() {
        let mut buffer = [0; MAX_PACKET_SIZE - 4];
        let mut packet = FirmwareBlock {
            length: 10,
            offset: 20,
            data: [0; FIRMWARE_BLOCK_MAX_DATA_SIZE],
        };

        for i in 0..10 {
            packet.data[i] = i as u8;
        }

        let command = Command::WriteFirmwareBlock(packet);

        let len = command.serialize(&mut buffer);
        let result = Command::deserialize(&buffer[..len]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), command);
    }

    #[test]
    fn finalize_firmware_update_command() {
        let mut buffer = [0; MAX_PACKET_SIZE];
        let command = Command::FinalizeFirmwareUpdate;
        let len = command.serialize(&mut buffer);
        let result = Command::deserialize(&buffer[..len]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Command::FinalizeFirmwareUpdate);
    }
}
