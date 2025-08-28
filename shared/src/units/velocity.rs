pub use crate::units::angle::Electrical;
pub use crate::units::angle::Mechanical;
use crate::units::low_pass_filter::LowPassFilter;
use core::marker::PhantomData;
use core::num::ParseIntError;
use core::str::FromStr;
use fixed::types::{I32F32, U1F15};

pub trait VelocityType {}

impl VelocityType for Electrical {}
impl VelocityType for Mechanical {}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Velocity<T: VelocityType> {
    raw_velocity: i32,
    _kind: PhantomData<T>,
}

#[cfg(feature = "defmt")]
impl defmt::Format for Velocity<Electrical> {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "Electrical {}rpm", self.as_rpm())
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for Velocity<Mechanical> {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "Mechanical {}rpm", self.as_rpm())
    }
}

impl Velocity<Electrical> {
    pub const ZERO: Self = Self {
        raw_velocity: 0,
        _kind: PhantomData,
    };
}

impl Velocity<Mechanical> {
    pub const ZERO: Self = Self {
        raw_velocity: 0,
        _kind: PhantomData,
    };
}

impl<T: VelocityType> Velocity<T> {
    const MULTIPLIER: i128 = 66;
    const SCALE: i128 = (u16::MAX as i128) * Self::MULTIPLIER / 60_000;

    /// 1 fraction is 1/u16::MAX of full rotation
    #[inline(always)]
    pub fn from_fraction_per_millisecond(val: i32) -> Self {
        Self {
            raw_velocity: val,
            _kind: PhantomData,
        }
    }

    #[inline(always)]
    pub fn from_rpm(val: i16) -> Self {
        Self::from_fraction_per_millisecond(((val as i128) * Self::SCALE / Self::MULTIPLIER) as i32)
    }

    #[inline(always)]
    pub fn as_rpm(&self) -> i16 {
        ((self.raw_velocity as i128) * Self::MULTIPLIER / Self::SCALE) as i16
    }
}

impl<T: VelocityType> FromStr for Velocity<T> {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_rpm(s.parse::<i16>()?))
    }
}

impl<T: VelocityType> LowPassFilter<Velocity<T>> for Velocity<T> {
    fn low_pass_filter(&self, value: Velocity<T>, alpha: U1F15) -> Velocity<T> {
        let self_velocity = I32F32::from_num(self.raw_velocity);
        let value_velocity = I32F32::from_num(value.raw_velocity);
        let mut change = self_velocity.low_pass_filter(value_velocity, alpha);
        // TODO improve resolution to fix this
        if change.abs() < 1 {
            change = change.signum() * 1;
        }
        Velocity {
            raw_velocity: (self_velocity - change).to_num::<i32>(),
            _kind: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_rpm() {
        assert_almost_eq(Velocity::<Electrical>::from_rpm(1000).as_rpm(), 1000);
        assert_almost_eq(Velocity::<Electrical>::from_rpm(2000).as_rpm(), 2000);
        assert_almost_eq(Velocity::<Electrical>::from_rpm(100).as_rpm(), 100);
        assert_almost_eq(
            Velocity::<Electrical>::from_rpm(i16::MAX).as_rpm(),
            i16::MAX,
        );
        assert_almost_eq(
            Velocity::<Electrical>::from_rpm(i16::MIN).as_rpm(),
            i16::MIN,
        );
    }

    #[test]
    fn test_from_str() {
        assert_almost_eq(
            Velocity::<Mechanical>::from_str("600").unwrap().as_rpm(),
            600,
        );
        assert_almost_eq(
            Velocity::<Mechanical>::from_str("2000").unwrap().as_rpm(),
            2000,
        );
        assert_almost_eq(
            Velocity::<Mechanical>::from_str("100").unwrap().as_rpm(),
            100,
        );
    }

    #[test]
    fn test_from_raw_per_millisecond() {
        assert_almost_eq(
            Velocity::<Mechanical>::from_fraction_per_millisecond(655).as_rpm(),
            600,
        );
    }

    fn assert_almost_eq(a: i16, b: i16) {
        assert!((a as i32 - b as i32).abs() <= 1, "{} != {}", a, b);
    }
}
