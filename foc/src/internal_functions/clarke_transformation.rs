use shared::fixed::types::I16F16;

const SQRT3_OVER_2: I16F16 = I16F16::lit("0.86602540378");

pub fn inverse(alpha: i16, beta: i16) -> (i16, i16, i16) {
    let alpha = I16F16::from(alpha);
    let beta = I16F16::from(beta);
    let a = alpha;
    let b = alpha.saturating_neg() / 2 + beta * SQRT3_OVER_2;
    let c = alpha.saturating_neg() / 2 - beta * SQRT3_OVER_2;

    (a.to_num(), b.to_num(), c.to_num())
}

#[cfg(test)]
mod tests {
    const FRAC_SQRT_3_2_Q15: i32 = 28377;
    use super::*;
    const TOLERANCE: i16 = 2;

    #[test]
    fn test_inverse_clarke_cardinals() {
        // alpha, beta, a, b, c
        let cases: &[(i16, i16, i16, i16, i16)] = &[
            (i16::MAX, 0, i16::MAX, -i16::MAX / 2, -i16::MAX / 2),
            (i16::MIN, 0, i16::MIN, i16::MAX / 2, i16::MAX / 2),
            (
                0,
                i16::MAX,
                0,
                FRAC_SQRT_3_2_Q15 as i16,
                -FRAC_SQRT_3_2_Q15 as i16,
            ),
            (
                0,
                i16::MIN,
                0,
                -FRAC_SQRT_3_2_Q15 as i16,
                FRAC_SQRT_3_2_Q15 as i16,
            ),
            (
                i16::MAX / 2,
                i16::MAX / 2,
                i16::MAX / 2,
                -i16::MAX / 4 + (FRAC_SQRT_3_2_Q15 as i16 / 2),
                -i16::MAX / 4 - (FRAC_SQRT_3_2_Q15 as i16 / 2),
            ),
            (
                -i16::MAX / 2,
                -i16::MAX / 2,
                -i16::MAX / 2,
                i16::MAX / 4 - (FRAC_SQRT_3_2_Q15 as i16 / 2),
                i16::MAX / 4 + (FRAC_SQRT_3_2_Q15 as i16 / 2),
            ),
        ];

        for &(alpha, beta, expected_a, expected_b, expected_c) in cases {
            let (a, b, c) = inverse(alpha, beta);

            assert!(
                (a - expected_a).abs() < TOLERANCE,
                "alpha={} beta={} => a={}, expected {} +- {}",
                alpha,
                beta,
                a,
                expected_a,
                TOLERANCE,
            );
            assert!(
                (b - expected_b).abs() < TOLERANCE,
                "alpha={} beta={} => b={}, expected {} +- {}",
                alpha,
                beta,
                b,
                expected_b,
                TOLERANCE,
            );
            assert!(
                (c - expected_c).abs() < TOLERANCE,
                "alpha={} beta={} => c={}, expected {} +- {}",
                alpha,
                beta,
                c,
                expected_c,
                TOLERANCE,
            );
        }
    }

    #[test]
    fn test_sum_of_currents_should_equal_zero() {
        let cases: &[(i16, i16)] = &[
            (i16::MAX / 2, i16::MAX / 2),
            (1212, 1111),
            (0, i16::MAX),
            (i16::MIN / 2, i16::MAX / 2),
        ];
        for &(alpha, beta) in cases {
            let (a, b, c) = inverse(alpha, beta);
            let sum = a as i32 + b as i32 + c as i32;
            assert!(
                sum.abs() <= TOLERANCE as i32,
                "alpha = {}, beta = {}, a={} b={} c={}, sum={}, expected 0 +- {}",
                alpha,
                beta,
                a,
                b,
                c,
                sum,
                TOLERANCE,
            );
        }
    }
}
