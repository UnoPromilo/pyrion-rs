use defmt::Format;

#[derive(Debug, Copy, Clone, Format, Eq, PartialEq)]
pub struct Resistance(i16);

impl Resistance {
    #[inline(always)]
    pub fn from_milliohms(val: i16) -> Self {
        Self(val)
    }

    #[inline(always)]
    pub fn as_milliohms(&self) -> i16 {
        self.0
    }
}
