use crate::units::cos_lut::COS_LUT;
use core::marker::PhantomData;
use core::str::FromStr;
use defmt::Formatter;
use fixed::ParseFixedError;
use fixed::types::{I1F15, U3F29, U16F16, U16F48};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Electrical;
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Mechanical;

pub trait AngleType {}

impl AngleType for Electrical {}
impl AngleType for Mechanical {}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]

pub struct Angle<T: AngleType> {
    raw_angle: u16,
    _kind: PhantomData<T>,
}

#[cfg(feature = "defmt")]
impl<T: AngleType> defmt::Format for Angle<T> {
    fn format(&self, fmt: Formatter) {
        defmt::write!(fmt, "Angle ({} °)", self.as_degrees().to_num::<f32>());
    }
}

#[derive(Debug, Copy, Clone)]
pub enum AngleAny {
    Electrical(Angle<Electrical>),
    Mechanical(Angle<Mechanical>),
}
#[cfg(feature = "defmt")]
impl defmt::Format for AngleAny {
    fn format(&self, fmt: Formatter) {
        match self {
            AngleAny::Electrical(value) => {
                defmt::write!(fmt, "Electrical ({}°)", value.as_degrees().to_num::<f32>(),)
            }
            AngleAny::Mechanical(value) => {
                defmt::write!(fmt, "Mechanical ({}°)", value.as_degrees().to_num::<f32>(),)
            }
        }
    }
}

impl<T: AngleType> Angle<T> {
    pub const fn from_raw(raw_angle: u16) -> Self {
        Self {
            raw_angle,
            _kind: PhantomData,
        }
    }

    pub const fn max() -> Self {
        Self {
            raw_angle: u16::MAX,
            _kind: PhantomData,
        }
    }

    pub const fn zero() -> Self {
        Self {
            raw_angle: 0,
            _kind: PhantomData,
        }
    }

    pub fn cos(&self) -> I1F15 {
        Self::cos_raw(self.raw_angle)
    }

    pub fn sin(&self) -> I1F15 {
        const FRAC_PI_2: u16 = u16::MAX / 4;
        Self::cos_raw(FRAC_PI_2.wrapping_sub(self.raw_angle))
    }

    fn cos_raw(raw: u16) -> I1F15 {
        let index = (raw >> 6) as usize;
        let fraction = raw & 0x3F;

        let a = COS_LUT[index];
        let b = COS_LUT[(index + 1) % COS_LUT.len()];

        let weight = I1F15::from_bits(((fraction as i32) << 9) as i16);
        a + (b - a) * weight
    }

    pub fn from_degrees(degrees: U16F16) -> Self {
        const MAX_DEGREES: U16F16 = U16F16::lit("360");
        debug_assert!(degrees < MAX_DEGREES);
        //  65535 / 360
        const SCALE: U16F16 = U16F16::lit("182.04166666667");
        let scaled = degrees * SCALE;
        Self {
            raw_angle: scaled.to_num::<u16>(),
            _kind: PhantomData,
        }
    }

    pub fn as_degrees(&self) -> U16F16 {
        // 360/65535
        const SCALE: U16F16 = U16F16::lit("0.005493247883");
        U16F16::from_num(self.raw_angle) * SCALE
    }

    pub fn from_rad(rad: U3F29) -> Self {
        const MAX_RAD: U3F29 = U3F29::lit("6.283185307179586477");
        debug_assert!(rad < MAX_RAD);
        // 65535 / 6.283185307179586477
        const SCALE: U16F48 = U16F48::lit("10430.21919552736082948");
        let scaled = rad.to_num::<U16F48>() * SCALE;
        Self {
            raw_angle: scaled.to_num::<u16>(),
            _kind: PhantomData,
        }
    }

    pub fn as_rad(&self) -> U3F29 {
        const SCALE: U16F48 = U16F48::lit("0.00009587526218");
        (U16F48::from_num(self.raw_angle) * SCALE).to_num()
    }

    pub fn checked_add(&self, other: &Self) -> Option<Self> {
        self.raw_angle.checked_add(other.raw_angle).map(|raw| Self {
            raw_angle: raw,
            _kind: PhantomData,
        })
    }

    pub fn checked_sub(&self, other: &Self) -> Option<Self> {
        self.raw_angle.checked_sub(other.raw_angle).map(|raw| Self {
            raw_angle: raw,
            _kind: PhantomData,
        })
    }

    pub fn overflowing_add(&self, other: &Self) -> Self {
        Self {
            raw_angle: self.raw_angle.overflowing_add(other.raw_angle).0,
            _kind: PhantomData,
        }
    }

    pub fn overflowing_sub(&self, other: &Self) -> Self {
        Self {
            raw_angle: self.raw_angle.overflowing_sub(other.raw_angle).0,
            _kind: PhantomData,
        }
    }

    pub fn inverted(&self) -> Self {
        Self {
            raw_angle: u16::MAX - self.raw_angle,
            _kind: PhantomData,
        }
    }

    pub fn raw(&self) -> u16 {
        self.raw_angle
    }
}

impl Angle<Electrical> {
    // TODO write this without if
    pub fn from_mechanical(
        angle: &Angle<Mechanical>,
        offset: &Angle<Electrical>,
        pole_pairs: i16,
    ) -> Self {
        if pole_pairs > 0 {
            let pole_pairs = pole_pairs as u16;
            Self::from_raw(
                angle
                    .raw_angle
                    .wrapping_sub(offset.raw_angle)
                    .wrapping_mul(pole_pairs),
            )
        } else {
            let pole_pairs = (-pole_pairs) as u16;
            Self::from_raw(
                angle
                    .raw_angle
                    .wrapping_sub(offset.raw_angle)
                    .wrapping_mul(pole_pairs),
            )
            .inverted()
        }
    }
}

impl<T: AngleType> FromStr for Angle<T> {
    type Err = ParseAngleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let degrees = U16F16::from_str(s)?;
        if degrees < 0 || degrees >= 360 {
            return Err(ParseAngleError::OutOfRange);
        }
        Ok(Self::from_degrees(degrees))
    }
}

pub enum ParseAngleError {
    ParseFixedError(ParseFixedError),
    OutOfRange,
}

impl From<ParseFixedError> for ParseAngleError {
    fn from(value: ParseFixedError) -> Self {
        ParseAngleError::ParseFixedError(value)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_sin_cos_variants() {
        let test_cases = [
            (U16F16::from_num(0), I1F15::MAX, I1F15::ZERO),
            (U16F16::from_num(90), I1F15::ZERO, I1F15::MAX),
            (U16F16::from_num(180), I1F15::MIN, I1F15::ZERO),
            (U16F16::from_num(270), I1F15::ZERO, I1F15::MIN),
            (U16F16::from_num(359.9999), I1F15::MAX, I1F15::ZERO),
        ];

        for &(angle, expected_cos, expected_sin) in &test_cases {
            let angle = Angle::<Electrical>::from_degrees(angle);
            let cos_val = angle.cos();
            let sin_val = angle.sin();

            assert!(
                approx_eq(cos_val, expected_cos),
                "cos failed for {}°: got {}, expected {}",
                angle.as_degrees(),
                cos_val,
                expected_cos
            );

            assert!(
                approx_eq(sin_val, expected_sin),
                "sin failed for {}: got {}, expected {}",
                angle.as_degrees(),
                sin_val,
                expected_sin
            );
        }
    }

    #[test]
    fn sin_cos_random() {
        let pi_over_2 = u16::MAX / 4;
        let raw_angle = u16::MAX / 8; // ~45°
        let angle = Angle::<Electrical>::from_raw(raw_angle);
        let complementary = Angle::<Electrical>::from_raw(pi_over_2.wrapping_sub(raw_angle));

        let sin_val = angle.sin();
        let cos_comp = complementary.cos();

        assert!(
            approx_eq(sin_val, cos_comp),
            "sin(x) should approx equal cos(π/2 - x): got {:?} vs {:?}",
            sin_val,
            cos_comp
        );
    }

    #[test]
    fn sin_cos_safety() {
        for raw_angle in (0..=u16::MAX).step_by(4096) {
            let a = Angle::<Electrical>::from_raw(raw_angle);
            let _ = a.cos();
            let _ = a.sin();
        }
    }

    #[test]
    fn test_angle_electrical_from_mechanical() {
        let cases = vec![
            (1000, 0, 1, 1000),
            (2000, 500, 1, 1500),
            (3000, 0, 3, 9000),
            (9300, 0, 7, 65100),
            (9400, 0, 7, 264),
            (9400, 10, 7, 194),
            (u16::MAX, 10, 2, u16::MAX - 21),
        ];

        for (raw_mech, offset, pole_pairs, expected) in cases {
            let offset = Angle::<Electrical>::from_raw(offset);
            let mechanical = Angle::<Mechanical>::from_raw(raw_mech);
            let electrical = Angle::<Electrical>::from_mechanical(&mechanical, &offset, pole_pairs);
            assert_eq!(
                electrical.raw_angle, expected,
                "Failed for mech={}, offset={}, pole_pairs={}",
                raw_mech, offset.raw_angle, pole_pairs
            );
        }
    }

    fn approx_eq(a: I1F15, b: I1F15) -> bool {
        let tolerance = I1F15::from_num(0.008);
        let diff = a.wrapping_dist(b);
        diff <= tolerance
    }
}
