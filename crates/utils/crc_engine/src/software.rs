use crate::CRC_POLY;
use crate::engine::CrcEngine;
use crc::{Algorithm, Crc};

pub struct SoftwareCrcEngine {
    crc: Crc<u16>,
}

impl SoftwareCrcEngine {
    pub fn new() -> Self {
        const ALGORITHM: Algorithm<u16> = Algorithm {
            width: 16,
            poly: CRC_POLY,
            init: 0xFFFF,
            refin: false,
            refout: false,
            xorout: 0x000,
            check: 0,
            residue: 0,
        };
        let crc = crc::Crc::<u16>::new(&ALGORITHM);
        Self { crc }
    }
}

impl CrcEngine for SoftwareCrcEngine {
    fn calculate(&mut self, data: &[u8]) -> u16 {
        self.crc.checksum(data)
    }
}
