pub struct Angle(pub u16);

impl Angle {
    pub fn from_degrees(degrees: u16) -> Self {
        debug_assert!(degrees < 360);
        Angle(((degrees as u32 * u16::MAX as u32) / 360) as u16)
    }
}