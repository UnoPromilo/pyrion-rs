use defmt::Format;
use crate::models::Direction;

#[derive(Format)]
pub struct Angle(u16);

impl Angle {
    /// Returns always the smallest distance between two angles, always positive, wraps 360->0
    pub fn get_abs(&self, other: &Self) -> Self {
        let diff = self
            .0
            .wrapping_sub(other.0)
            .min(other.0.wrapping_sub(self.0));
        Angle(diff)
    }

    pub fn get_direction(&self, other: &Self) -> Option<Direction> {
        let diff = other.0.wrapping_sub(self.0);

        const HALF: u16 = u16::MAX / 2;

        match diff {
            0 => None,
            1..HALF => Some(Direction::CounterClockwise),
            HALF..=u16::MAX => Some(Direction::Clockwise),
        }
    }

    pub fn get_raw(&self) -> u16 {
        self.0
    }

    pub fn from_raw(raw: u16) -> Self {
        Angle(raw)
    }

    // to use only in tests
    pub fn from_degrees(degrees: u16) -> Self {
        debug_assert!(degrees < 360);
        Angle(((degrees as u32 * u16::MAX as u32) / 360) as u16)
    }

    // to use only in tests
    pub fn to_degrees(&self) -> u16 {
        ((self.0 as u32 * 360) / u16::MAX as u32) as u16
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use alloc::vec;

    const TOLERANCE: u16 = 1;

    fn assert_close(expected: u16, actual: u16) {
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
            let a1 = Angle::from_degrees(from);
            let a2 = Angle::from_degrees(to);
            let actual1 = a1.get_abs(&a2).to_degrees();
            let actual2 = a2.get_abs(&a1).to_degrees();
            assert_close(expected, actual1);
            assert_close(expected, actual2);
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
            let from = Angle::from_degrees(from_deg);
            let to = Angle::from_degrees(to_deg);
            let result = from.get_direction(&to);
            assert_eq!(
                result, expected,
                "Angle::get_direction({from_deg}, {to_deg}) = {:?}, expected {:?}",
                result, expected
            );
        }
    }
}
