use crate::units::Direction;
use crate::units::cos_lut::COS_LUT;
use core::marker::PhantomData;
use core::str::FromStr;
use defmt::Format;
use fixed::ParseFixedError;
use fixed::types::{I1F15, U16F16};

#[derive(Debug, Copy, Clone)]
pub struct Electrical;
#[derive(Debug, Copy, Clone)]
pub struct Mechanical;

pub trait AngleType {}

impl AngleType for Electrical {}
impl AngleType for Mechanical {}

#[derive(Debug, Copy, Clone, Format, Eq, PartialEq)]

pub struct Angle<T: AngleType> {
    raw_angle: u16,
    _kind: PhantomData<T>,
}

#[derive(Debug, Copy, Clone, Format)]
pub enum AngleAny {
    Electrical(Angle<Electrical>),
    Mechanical(Angle<Mechanical>),
}

impl<T: AngleType> Angle<T> {
    pub fn from_raw(raw_angle: u16) -> Self {
        Self {
            raw_angle,
            _kind: PhantomData,
        }
    }

    /// Returns always the smallest distance between two angles, always positive, wraps 360->0
    pub fn get_abs(&self, other: &Self) -> Self {
        let diff = self.raw_angle.wrapping_sub(other.raw_angle);
        let dist = diff.min(u16::MAX - diff);
        Self {
            raw_angle: dist,
            _kind: PhantomData,
        }
    }

    pub fn get_direction(&self, other: &Self) -> Option<Direction> {
        let diff = other.raw_angle.wrapping_sub(self.raw_angle);

        const HALF: u16 = u16::MAX / 2;

        match diff {
            0 => None,
            1..HALF => Some(Direction::CounterClockwise),
            HALF..=u16::MAX => Some(Direction::Clockwise),
        }
    }

    pub fn cos(&self) -> I1F15 {
        let cos_q = (self.raw_angle >> 6) as usize;
        COS_LUT[cos_q]
    }

    pub fn sin(&self) -> I1F15 {
        const FRAC_PI_2: u16 = u16::MAX / 4;
        let sin_q = (FRAC_PI_2.wrapping_sub(self.raw_angle) >> 6) as usize;
        COS_LUT[sin_q]
    }

    pub fn from_degrees(degrees: U16F16) -> Self {
        let max_degrees = U16F16::from_num(360);
        debug_assert!(degrees < max_degrees);
        // u16::MAX = 65535, so scale = 65535 / 360 = 182.041...
        // In U16F16 representation: 182.041... * 2^16 ≈ 11930283
        const SCALE: U16F16 = U16F16::from_bits(11930283);
        let scaled = degrees * SCALE;
        Self {
            raw_angle: scaled.to_num::<u16>(),
            _kind: PhantomData,
        }
    }

    pub fn as_degrees(&self) -> U16F16 {
        // u16::MAX = 65535, so scale = 360/65535 = 0.00549...
        // In U16F16 representation: 0.00549... * 2^16 ≈ 360
        const SCALE: U16F16 = U16F16::from_bits(360);
        U16F16::from_num(self.raw_angle) * SCALE
    }
}

impl Angle<Electrical> {
    pub fn from(angle: &Angle<Mechanical>, offset: u16, pole_pairs: u16) -> Self {
        Self::from_raw(
            angle
                .raw_angle
                .wrapping_sub(offset)
                .wrapping_mul(pole_pairs),
        )
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
    fn test_get_difference() {
        let cases = vec![
            (0, 20, 20),
            (20, 10, 10),
            (340, 20, 40),
            (20, 345, 35),
            (359, 359, 0),
        ];

        for (from, to, expected) in cases {
            let a1 = Angle::<Electrical>::from_degrees(U16F16::from_num(from));
            let a2 = Angle::from_degrees(U16F16::from_num(to));
            let actual1 = a1.get_abs(&a2).as_degrees();
            let actual2 = a2.get_abs(&a1).as_degrees();
            assert_close(expected, actual1.to_num());
            assert_close(expected, actual2.to_num());
        }
    }

    #[test]
    fn test_get_direction() {
        use Direction::*;

        let cases = vec![
            (0, 0, None),
            (0, 1, Some(CounterClockwise)),
            (0, 179, Some(CounterClockwise)),
            (0, 180, Some(Clockwise)), // exactly 180°, defined as CW in impl
            (0, 181, Some(Clockwise)),
            (359, 0, Some(CounterClockwise)), // wraparound
            (0, 359, Some(Clockwise)),        // wraparound the other way
            (90, 270, Some(Clockwise)),
            (270, 90, Some(Clockwise)), // exactly 180°, defined as CW in impl
        ];

        for (from_deg, to_deg, expected) in cases {
            let from = Angle::<Mechanical>::from_degrees(U16F16::from_num(from_deg));
            let to = Angle::from_degrees(U16F16::from_num(to_deg));
            let result = from.get_direction(&to);
            assert_eq!(
                result, expected,
                "Angle::get_direction({from_deg}, {to_deg}) = {:?}, expected {:?}",
                result, expected
            );
        }
    }

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
            let mechanical = Angle::<Mechanical>::from_raw(raw_mech);
            let electrical = Angle::<Electrical>::from(&mechanical, offset, pole_pairs);
            assert_eq!(
                electrical.raw_angle, expected,
                "Failed for mech={}, offset={}, pole_pairs={}",
                raw_mech, offset, pole_pairs
            );
        }
    }

    fn approx_eq(a: I1F15, b: I1F15) -> bool {
        let tolerance = I1F15::from_num(0.008);
        let diff = a.wrapping_dist(b);
        diff <= tolerance
    }

    fn assert_close(expected: u16, actual: u16) {
        const TOLERANCE: u16 = 1;
        let diff = (expected as i32 - actual as i32).abs();
        assert!(
            diff <= TOLERANCE as i32,
            "expected {}, got {}, diff {} > tolerance {}",
            expected,
            actual,
            diff,
            TOLERANCE
        );
    }
}
