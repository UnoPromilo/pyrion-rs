use crate::modules::models::ElectricalAngle;

/// Performs the inverse Park transformation.
///
/// Converts rotating D/Q-axis current vector components (`i_d`, `i_q`) back into
/// stationary α/β (2-phase) reference frame components using the provided electrical angle.
///
/// # Input
/// - `i_d`: Direct-axis (D) current component, Q15-scaled, range: `i16::MIN` to `i16::MAX`
/// - `i_q`: Quadrature-axis (Q) current component, Q15-scaled, range: `i16::MIN` to `i16::MAX`
/// - `electrical_angle`: Electrical rotor angle, fixed-point representation (`ElectricalAngle`)
///
/// Internally, sine and cosine of the angle are evaluated using Q15 format (i16),
/// and all calculations are performed in 32-bit to avoid intermediate overflow.
///
/// # Returns
/// Tuple `(alpha, beta)` where:
/// - `alpha`: α-axis current component (Q15 format)
/// - `beta`: β-axis current component (Q15 format)
/// - Output range: full `i16` range (±32,768), precision depends on input magnitude and angle
///
/// # Notes
/// - The function assumes normalized, balanced input values
/// - Uses fixed-point arithmetic (`Q15`) throughout
/// - Internally applies right-shift by 15 to scale down 32-bit intermediate results
/// - This function is deterministic and does not panic or overflow
pub fn inverse(i_d: i16, i_q: i16, electrical_angle: &ElectricalAngle) -> (i16, i16) {
    let sin_theta = electrical_angle.sin_q15();
    let cos_theta = electrical_angle.cos_q15();
    let alpha = ((i_d as i32 * cos_theta as i32 - i_q as i32 * sin_theta as i32) >> 15) as i16;
    let beta = ((i_d as i32 * sin_theta as i32 + i_q as i32 * cos_theta as i32) >> 15) as i16;

    (alpha, beta)
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::f32::consts::FRAC_1_SQRT_2;
    use hardware_abstraction::models::Angle;

    const TOLERANCE: i16 = 3;

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
            let angle = ElectricalAngle::from_angle(&Angle::from_degrees(deg), 0, 1);
            test_with_values(&angle, expected_alpha, expected_beta);
        }
    }

    #[test]
    fn test_zero_input() {
        let angle = ElectricalAngle::from_angle(&Angle::from_degrees(123), 0, 1);
        let (alpha, beta) = inverse(0, 0, &angle);
        assert_eq!(alpha, 0);
        assert_eq!(beta, 0);
    }

    fn test_with_values(angle: &ElectricalAngle, expected_alpha: i16, expected_beta: i16) {
        let (i_alpha, i_beta) = inverse(i16::MAX, 0, angle);
        assert_close(expected_alpha, i_alpha);
        assert_close(expected_beta, i_beta);
    }
    fn assert_close(expected: i16, actual: i16) {
        assert!(
            (expected - actual).abs() <= TOLERANCE,
            "expected {}, got {}, diff {} > tolerance {}",
            expected,
            actual,
            (expected - actual).abs(),
            TOLERANCE
        );
    }
}
