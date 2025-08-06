use core::num::ParseIntError;
use core::ops::{Neg, Sub};
use core::str::FromStr;
use defmt::Format;

#[derive(Debug, Copy, Clone, Format, Eq, PartialEq)]
pub struct Current(i16);

impl Current {
    #[inline(always)]
    pub fn from_milliamps(val: i16) -> Self {
        Self(val)
    }

    #[inline(always)]
    pub fn as_milliamps(&self) -> i16 {
        self.0
    }
}

impl Neg for Current {
    type Output = Current;

    #[inline(always)]
    fn neg(self) -> Self::Output {
        Current(-self.0)
    }
}

impl Sub for Current {
    type Output = Current;

    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        Current(self.0 - rhs.0)
    }
}

impl FromStr for Current {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_milliamps(s.parse::<i16>()?))
    }
}

