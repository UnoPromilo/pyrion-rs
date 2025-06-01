pub const ADDRESS: u8 = 0x36;

#[derive(Debug)]
#[allow(dead_code)]
pub enum Register {
    RawAngle = 0x0C,
    Angle = 0x0E,
    Agc = 0x1A,
    Magnitude = 0x1B,
    ConfHigh = 0x07,
    ConfLow = 0x08,
    ZPosHigh = 0x01,
    ZPosLow = 0x02,
    MPosHigh = 0x03,
    MPosLow = 0x04,
}
impl Into<u8> for Register {
    fn into(self) -> u8 {
        self as u8
    }
}
