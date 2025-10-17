#[derive(Debug, Eq, PartialEq)]
pub enum Command {
    Ping, // 0x01

    Reset, // 0xEE
}

impl Command {
    pub fn deserialize(buffer: &[u8]) -> Result<Self, Error> {
        let cmd_byte = buffer[0];
        match cmd_byte {
            0x01 => Ok(Command::Ping),
            0xEE => Ok(Command::Reset),
            _ => Err(Error::CommandNotFound),
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
    fn ping_command() {
        let buffer = [0x01];
        let result = Command::deserialize(&buffer);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Command::Ping);
    }
}
