pub trait CrcEngine {
    fn calculate(&mut self, data: &[u8]) -> u16;

    fn check(&mut self, data: &[u8]) -> bool {
        assert!(data.len() > 2, "Data too short");
        let calculated_crc = self.calculate(&data[..data.len() - 2]);
        let received_crc = u16::from_le_bytes([data[data.len() - 2], data[data.len() - 1]]);
        calculated_crc == received_crc
    }
}
