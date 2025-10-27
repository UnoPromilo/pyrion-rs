use crate::CRC_POLY;
use crate::engine::CrcEngine;
use core::fmt::Debug;
use crc::{Algorithm, Crc};

pub struct SoftwareCrcEngine {
    crc: Crc<u16>,
}

impl Debug for SoftwareCrcEngine {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SoftwareCrcEngine")
    }
}

impl Default for SoftwareCrcEngine {
    fn default() -> Self {
        Self::new()
    }
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
