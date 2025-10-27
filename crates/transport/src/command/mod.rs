use crate::packet::Packet;

pub mod decoder;
pub mod encoder;
#[derive(Debug, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Command {
    IntroduceYourself, // 0x01
    Stop,              // 0x02
}

impl Packet for Command {
    type Error = Error;

    fn deserialize(data: &[u8]) -> Result<Self, Self::Error> {
        let cmd_byte = data[0];
        match cmd_byte {
            0x01 => Ok(Command::IntroduceYourself),
            0x02 => Ok(Command::Stop),
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
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    CommandNotFound,
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
