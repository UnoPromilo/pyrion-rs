use core::num::ParseIntError;
use core::str::FromStr;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Voltage(i16);

impl Voltage {
    #[inline(always)]
    pub fn from_millivolts(val: i16) -> Self {
        Self(val)
    }

    #[inline(always)]
    pub fn as_millivolts(&self) -> i16 {
        self.0
    }
}

impl FromStr for Voltage {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_millivolts(s.parse::<i16>()?))
    }
}
