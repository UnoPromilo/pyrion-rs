#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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
