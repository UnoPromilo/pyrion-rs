use core::num::ParseIntError;
use core::str::FromStr;
use defmt::Format;

#[derive(Debug, Copy, Clone, Format, Eq, PartialEq)]
pub struct Velocity(i16);

impl Velocity {
    #[inline(always)]
    pub fn from_rpm(val: i16) -> Self {
        Self(val)
    }

    #[inline(always)]
    pub fn as_rpm(&self) -> i16 {
        self.0
    }
}

impl FromStr for Velocity {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_rpm(s.parse::<i16>()?))
    }
}
