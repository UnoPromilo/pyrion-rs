use crate::command::Command;
use crate::decoder;

pub type Decoder = decoder::Decoder<Command, 0xAA>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MAX_PACKET_SIZE;
    use crate::decoder::DecoderError;
    use crc_engine::CrcEngine;
    use crc_engine::software::SoftwareCrcEngine;

    #[test]
    fn introduce_yourself_command_should_be_recognized() {
        let buffer = {
            let mut buffer = [0xAA, 0x01, 0x01, 0x00, 0x00];
            calculate_and_attach_crc(&mut buffer);
            buffer
        };
        let mut parser = Decoder::new();
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
                assert_eq!(command, Command::IntroduceYourself);
            }
        }
    }

    #[test]
    fn invalid_crc_should_be_recognized() {
        let buffer = [0xAA, 0x01, 0x01, 0x00, 0x00];
        let mut parser = Decoder::new();
        let mut crc = SoftwareCrcEngine::new();
        for i in 0..buffer.len() {
            let result = parser.feed(buffer[i], &mut crc);
            if i == buffer.len() - 1 {
                let error = result.unwrap().unwrap_err();
                assert_eq!(error, DecoderError::InvalidCrc);
            }
        }
    }

    #[test]
    fn max_length_message_should_be_parsed() {
        let buffer = {
            let mut buffer = [0; MAX_PACKET_SIZE];
            buffer[0] = 0xAA;
            buffer[1] = u8::MAX;
            buffer[2] = 0x01; // IntroduceYourself command
            calculate_and_attach_crc(&mut buffer);
            buffer
        };
        let mut crc = SoftwareCrcEngine::new();
        let mut parser = Decoder::new();
        for i in 0..buffer.len() - 1 {
            let _ = parser.feed(buffer[i], &mut crc);
        }
        let result = parser.feed(buffer[buffer.len() - 1], &mut crc);
        let command = result.unwrap().unwrap();
        assert_eq!(command, Command::IntroduceYourself);
    }

    fn calculate_and_attach_crc(buffer: &mut [u8]) {
        let mut crc = SoftwareCrcEngine::new();
        let crc = crc.calculate(&buffer[..buffer.len() - 2]);
        let crc_bytes = crc.to_le_bytes();
        buffer[buffer.len() - 2] = crc_bytes[0];
        buffer[buffer.len() - 1] = crc_bytes[1];
    }
}
