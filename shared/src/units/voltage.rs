use defmt::Format;

#[derive(Debug, Copy, Clone, Format, Eq, PartialEq)]
pub struct Voltage(i16);

impl Voltage {
    #[inline(always)]
    pub fn from_millivolts(val: i16) -> Self {
        Self(val)
    }

    #[inline(always)]
    pub fn as_millivolts(self) -> i16 {
        self.0
    }
}
