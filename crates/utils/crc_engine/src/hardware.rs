use crate::CRC_POLY;
use crate::engine::CrcEngine;
use embassy_stm32::crc::{InputReverseConfig, PolySize};
use embassy_stm32::{Peri, crc, peripherals};

pub struct HardwareCrcEngine<'a> {
    crc: crc::Crc<'a>,
}

impl<'a> HardwareCrcEngine<'a> {
    pub fn new(peri: Peri<'a, peripherals::CRC>) -> Self {
        let config = crc::Config::new(
            InputReverseConfig::None,
            false,
            PolySize::Width16,
            0xFFFF,
            CRC_POLY as u32,
        )
        .expect("Invalid CRC config");
        Self {
            crc: crc::Crc::new(peri, config),
        }
    }
}

impl<'a> CrcEngine for HardwareCrcEngine<'a> {
    fn calculate(&mut self, data: &[u8]) -> u16 {
        self.crc.reset();
        let value = self.crc.feed_bytes(data);
        (value & 0xFFFF) as u16
    }
}
