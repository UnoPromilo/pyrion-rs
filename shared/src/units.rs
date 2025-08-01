use core::ops::{Neg, Sub};
use defmt::Format;

#[derive(Debug, Copy, Clone, Format, Eq, PartialEq)]
pub struct Current(i16);

#[derive(Debug, Copy, Clone, Format, Eq, PartialEq)]
pub struct Voltage(i16);

#[derive(Debug, Copy, Clone, Format, Eq, PartialEq)]
pub struct Resistance(i16);

impl Current {
    #[inline(always)]
    pub fn from_milliamps(val: i16) -> Self {
        Self(val)
    }

    #[inline(always)]
    pub fn as_milliamps(self) -> i16 {
        self.0
    }
}

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

impl Resistance {
    #[inline(always)]
    pub fn from_milliohms(val: i16) -> Self {
        Self(val)
    }

    #[inline(always)]
    pub fn as_milliohms(self) -> i16 {
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
