use crate::command::Command;
use crc_engine::CrcEngine;

const MAX_PACKET_SIZE: usize = (u8::MAX as usize) + 4;

const START_BYTE: u8 = 0xAA;
// Struct of message = [START(u8)][LENGTH(u8)][CMD_ID(u8)][PAYLOAD][CRC(u16)]

pub struct Parser {
    position: usize,
    buffer: [u8; MAX_PACKET_SIZE],
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser {
    pub fn new() -> Self {
        Self {
            position: 0,
            buffer: [0; MAX_PACKET_SIZE],
        }
    }

    pub fn feed(&mut self, data: u8, crc: &mut impl CrcEngine) -> Option<Result<Command, Error>> {
        if self.position == 0 && data != START_BYTE {
            return None;
        }

        self.buffer[self.position] = data;
        self.position += 1;

        if self.position > 2 {
            let length = self.length();
            if self.position == length + 4 {
                let parsing_result = self.parse_command(crc);
                self.position = 0;

                return Some(parsing_result);
            }
        }

        None
    }

    fn parse_command(&self, crc: &mut impl CrcEngine) -> Result<Command, Error> {
        if !crc.check(&self.buffer[..self.length() + 4]) {
            return Err(Error::InvalidCrc);
        }
        // Crop first two bytes and last two bytes
        //
        let cmd = Command::deserialize(&self.buffer[2..self.length() + 2])?;
        Ok(cmd)
    }

    fn length(&self) -> usize {
        self.buffer[1] as usize
    }
}

#[derive(Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    InvalidCrc,
    DeserializeError(crate::command::Error),
}

impl From<crate::command::Error> for Error {
    fn from(error: crate::command::Error) -> Self {
        Error::DeserializeError(error)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crc_engine::software::SoftwareCrcEngine;

    #[test]
    fn ping_command_should_be_recognized() {
        let buffer = {
            let mut buffer = [0xAA, 0x01, 0x01, 0x00, 0x00];
            calculate_and_attach_crc(&mut buffer);
            buffer
        };
        let mut parser = Parser::new();
        let mut crc = SoftwareCrcEngine::new();
        for i in 0..buffer.len() {
            let result = parser.feed(buffer[i], &mut crc);
            if i < buffer.len() - 1 {
                assert!(result.is_none());
            } else {
                assert!(result.is_some());
                let some = result.unwrap();
                assert!(some.is_ok());
                let command = some.unwrap();
                assert_eq!(command, Command::Ping);
            }
        }
    }

    #[test]
    fn invalid_crc_should_be_recognized() {
        let buffer = [0xAA, 0x01, 0x01, 0x00, 0x00];
        let mut parser = Parser::new();
        let mut crc = SoftwareCrcEngine::new();
        for i in 0..buffer.len() {
            let result = parser.feed(buffer[i], &mut crc);
            if i == buffer.len() - 1 {
                let error = result.unwrap().unwrap_err();
                assert_eq!(error, Error::InvalidCrc);
            }
        }
    }

    #[test]
    fn max_length_message_should_be_parsed() {
        let buffer = {
            let mut buffer = [0; MAX_PACKET_SIZE];
            buffer[0] = START_BYTE;
            buffer[1] = u8::MAX;
            buffer[2] = 0x01; // Ping command
            calculate_and_attach_crc(&mut buffer);
            buffer
        };
        let mut crc = SoftwareCrcEngine::new();
        let mut parser = Parser::new();
        for i in 0..buffer.len() - 1 {
            let _ = parser.feed(buffer[i], &mut crc);
        }
        let result = parser.feed(buffer[buffer.len() - 1], &mut crc);
        let command = result.unwrap().unwrap();
        assert_eq!(command, Command::Ping);
    }

    fn calculate_and_attach_crc(buffer: &mut [u8]) {
        let mut crc = SoftwareCrcEngine::new();
        let crc = crc.calculate(&buffer[..buffer.len() - 2]);
        let crc_bytes = crc.to_le_bytes();
        buffer[buffer.len() - 2] = crc_bytes[0];
        buffer[buffer.len() - 1] = crc_bytes[1];
    }
}
