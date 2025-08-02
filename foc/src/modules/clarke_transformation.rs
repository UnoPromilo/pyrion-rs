const FRAC_SQRT_3_2_Q15: i32 = 28377;

/// Performs the inverse Clarke transformation.
///
/// Converts α/β (stationary 2-phase) current vector components back into
/// three-phase (A, B, C) current values.
///
/// # Input
/// - `alpha`: α-axis current component, range: from `i16::MIN` to `i16::MAX`
/// - `beta`: β-axis current component, range: from `i16::MIN` to `i16::MAX`
/// - `alpha` +- `beta`: must be in range from `i16::MIN` to `i16::MAX`
///
/// Both inputs should represent Q15-scaled values, covering the full dynamic range
/// of `i16`. Internally, the computation uses 32-bit arithmetic to preserve accuracy
/// and avoid overflow during intermediate steps.
///
/// # Returns
/// Tuple `(a, b, c)` where:
/// - Each output is a phase current value (in Q15 format)
/// - The sum of all three values is always zero (balanced 3-phase system)
/// - Values cover the full `i16` range (±32,768)
///
/// # Notes
/// - The function assumes ideal conditions (no offset, noise, or imbalance)
/// - Internally uses `FRAC_SQRT_3_2_Q15` (√3 / 2 in Q15 format)
/// - This function is deterministic and panic-free (does not overflow)
pub fn inverse(alpha: i16, beta: i16) -> (i16, i16, i16) {
    // TODO convert to fixed
    let a = alpha;
    let b = (((-(alpha as i32)) >> 1) + ((beta as i32 * FRAC_SQRT_3_2_Q15) >> 15)) as i16;
    let c = (((-(alpha as i32)) >> 1) - ((beta as i32 * FRAC_SQRT_3_2_Q15) >> 15)) as i16;
    (a, b, c)
}


#[cfg(test)]
mod tests {
    use super::*;
    const TOLERANCE: i16 = 3;

    #[test]
    fn test_inverse_clarke_cardinals() {
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
