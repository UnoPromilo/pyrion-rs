use fixed::types::I32F32;
use shared::units::angle::Electrical;
use shared::units::Angle;

pub fn forward(alpha: i32, beta: i32, electrical_angle: &Angle<Electrical>) -> (i32, i32) {
    let sin_theta = electrical_angle.sin().to_num::<I32F32>();
    let cos_theta = electrical_angle.cos().to_num::<I32F32>();

    let alpha = I32F32::from_num(alpha);
    let beta = I32F32::from_num(beta);

    let d = (alpha * cos_theta + beta * sin_theta).to_num::<i32>();
    let q = (-alpha * sin_theta + beta * cos_theta).to_num::<i32>();

    (d, q)
}

pub fn inverse(i_d: i32, i_q: i32, electrical_angle: &Angle<Electrical>) -> (i32, i32) {
    let sin_theta = electrical_angle.sin().to_num::<I32F32>();
    let cos_theta = electrical_angle.cos().to_num::<I32F32>();

    let i_d = I32F32::from_num(i_d);
    let i_q = I32F32::from_num(i_q);

    let alpha = (i_d * cos_theta - i_q * sin_theta).to_num::<i32>();
    let beta = (i_d * sin_theta + i_q * cos_theta).to_num::<i32>();

    (alpha, beta)
}

// TODO write better tests
/*
#[cfg(test)]
mod tests {
    use super::*;
    use core::f32::consts::FRAC_1_SQRT_2;
    use fixed::types::U16F16;

    const TOLERANCE: i64 = 4;

    #[test]
    fn test_inverse_park_cardinals() {
        const FRAC_1_SQRT_2_Q15: i16 = (i16::MAX as f32 * FRAC_1_SQRT_2) as i16;
        let cases = [
            (0, i16::MAX, 0),
            (45, FRAC_1_SQRT_2_Q15, FRAC_1_SQRT_2_Q15),
            (90, 0, i16::MAX),
            (135, -FRAC_1_SQRT_2_Q15, FRAC_1_SQRT_2_Q15),
            (180, i16::MIN, 0),
            (225, -FRAC_1_SQRT_2_Q15, -FRAC_1_SQRT_2_Q15),
            (270, 0, i16::MIN),
            (315, FRAC_1_SQRT_2_Q15, -FRAC_1_SQRT_2_Q15),
        ];

        for (deg, expected_alpha, expected_beta) in cases {
            let angle = Angle::<Electrical>::from_degrees(U16F16::from_num(deg));
            test_with_values(&angle, expected_alpha as i32, expected_beta as i32);
        }
    }

    #[test]
    fn test_zero_input() {
        let angle = Angle::<Electrical>::from_degrees(U16F16::from_num(123));
        let (alpha, beta) = inverse(0, 0, &angle);
        assert_eq!(alpha, 0);
        assert_eq!(beta, 0);
    }

    fn test_with_values(angle: &Angle<Electrical>, expected_alpha: i32, expected_beta: i32) {
        let (i_alpha, i_beta) = inverse(0, i32::MAX, angle);
        assert_close(expected_alpha, i_alpha);
        assert_close(expected_beta, i_beta);
    }
    fn assert_close(expected: i32, actual: i32) {
        assert!(
            (expected as i64 - actual as i64).abs() <= TOLERANCE,
            "expected {}, got {}, diff {} > tolerance {}",
            expected,
            actual,
            (expected as i64 - actual as i64).abs(),
            TOLERANCE
        );
    }
}
*/