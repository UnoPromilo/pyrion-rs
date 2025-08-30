use fixed::types::I32F32;

const SQRT3_OVER_2: I32F32 = I32F32::lit("0.866025403784438646763723170752936183");
const TWO_OVER_3: I32F32 = I32F32::lit("0.666666666666666666666666666666666667");
const ONE_OVER_3: I32F32 = I32F32::lit("0.333333333333333333333333333333333333");
const ONE_OVER_SQRT3: I32F32 = I32F32::lit("0.577350269189625764509148780501957456");
const TWO_OVER_SQRT3: I32F32 = I32F32::lit("1.154700538379251529018297561003914911");

pub fn inverse(alpha: i32, beta: i32) -> (i32, i32, i32) {
    let alpha = I32F32::from(alpha);
    let beta = I32F32::from(beta);
    let a = alpha;
    let b = alpha.saturating_neg() / 2 + beta * SQRT3_OVER_2;
    let c = alpha.saturating_neg() / 2 - beta * SQRT3_OVER_2;

    (a.to_num(), b.to_num(), c.to_num())
}

pub fn full(a: i32, b: i32, c: i32) -> (i32, i32) {
    let a = I32F32::from(a);
    let b = I32F32::from(b);
    let c = I32F32::from(c);

    let alpha = (TWO_OVER_3 * a) - (ONE_OVER_3 * b) - (ONE_OVER_3 * c);
    let beta = (ONE_OVER_SQRT3 * b) - (ONE_OVER_SQRT3 * c);

    (alpha.to_num(), beta.to_num())
}

pub fn assume_balanced(a: i32, b: i32) -> (i32, i32) {
    let a = I32F32::from(a);
    let b = I32F32::from(b);

    let alpha = a;
    let beta = (ONE_OVER_SQRT3 * a) + (TWO_OVER_SQRT3 * b);

    (alpha.to_num(), beta.to_num())
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::i32;
    const TOLERANCE: i32 = 2;

    #[test]
    fn test_inverse_clarke_cardinals() {
        let frac_sqrt_3_2: i32 = (SQRT3_OVER_2 * I32F32::MAX).to_num();
        // alpha, beta, a, b, c
        let cases: &[(i32, i32, i32, i32, i32)] = &[
            (i32::MAX, 0, i32::MAX, -i32::MAX / 2, -i32::MAX / 2),
            (i32::MIN, 0, i32::MIN, i32::MAX / 2, i32::MAX / 2),
            (0, i32::MAX, 0, frac_sqrt_3_2, -frac_sqrt_3_2),
            (0, i32::MIN, 0, -frac_sqrt_3_2, frac_sqrt_3_2),
            (
                i32::MAX / 2,
                i32::MAX / 2,
                i32::MAX / 2,
                -i32::MAX / 4 + (frac_sqrt_3_2 / 2),
                -i32::MAX / 4 - (frac_sqrt_3_2 / 2),
            ),
            (
                -i32::MAX / 2,
                -i32::MAX / 2,
                -i32::MAX / 2,
                i32::MAX / 4 - (frac_sqrt_3_2 / 2),
                i32::MAX / 4 + (frac_sqrt_3_2 / 2),
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
        let cases: &[(i32, i32)] = &[
            (i32::MAX / 2, i32::MAX / 2),
            (1212, 1111),
            (0, i32::MAX),
            (i32::MIN / 2, i32::MAX / 2),
        ];
        for &(alpha, beta) in cases {
            let (a, b, c) = inverse(alpha, beta);
            let sum = a + b + c;
            assert!(
                sum.abs() <= TOLERANCE,
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

    #[test]
    fn test_full_vs_manual_formula() {
        let cases: &[(i32, i32, i32)] = &[
            (30_000, 0, -30_000),
            (20_000, 10_000, -30_000),
            (10_000, -15_000, 5_000),
            (-25_000, 5_000, 20_000),
            (12_345, -23_456, 11_111),
        ];

        for &(a, b, c) in cases {
            let (alpha_full, beta_full) = full(a, b, c);
            let (alpha_balanced, beta_balanced) = assume_balanced(a, b);

            let expected_alpha = ((2 * a) - b - c) / 3;
            let expected_beta = ((b - c) as f32) / 3f32.sqrt();

            assert!(
                (alpha_full - expected_alpha).abs() <= TOLERANCE,
                "a={}, b={}, c={} => alpha_full={}, expected {}",
                a,
                b,
                c,
                alpha_full,
                expected_alpha
            );
            assert!(
                (alpha_balanced - expected_alpha).abs() <= TOLERANCE,
                "a={}, b={}, c={} => alpha_balanced={}, expected {}",
                a,
                b,
                c,
                alpha_balanced,
                expected_alpha
            );
            assert!(
                ((beta_full as f32) - expected_beta).abs() <= TOLERANCE as f32,
                "a={}, b={}, c={} => beta_full={}, expected {}",
                a,
                b,
                c,
                beta_full,
                expected_beta
            );
            assert!(
                ((beta_balanced as f32) - expected_beta).abs() <= TOLERANCE as f32,
                "a={}, b={}, c={} => beta_balanced={}, expected {}",
                a,
                b,
                c,
                beta_balanced,
                expected_beta
            );
        }
    }

    #[test]
    fn test_assume_balanced_matches_full() {
        let cases = &[(1000, -500), (200, 200), (-321, 654)];
        for &(a, b) in cases {
            let c = -a - b;
            let (alpha1, beta1) = full(a, b, c);
            let (alpha2, beta2) = assume_balanced(a, b);

            assert!(
                (alpha1 - alpha2).abs() <= TOLERANCE,
                "a={}, b={}, c={} => full.alpha={} balanced.alpha={}",
                a,
                b,
                c,
                alpha1,
                alpha2
            );
            assert!(
                (beta1 - beta2).abs() <= TOLERANCE,
                "a={}, b={}, c={} => full.beta={} balanced.beta={}",
                a,
                b,
                c,
                beta1,
                beta2
            );
        }
    }

    #[test]
    fn test_inverse_of_full_is_identity() {
        let cases = &[(1000, -500, -500), (300, 400, -700), (-111, 222, -111)];
        for &(a, b, c) in cases {
            let (alpha, beta) = full(a, b, c);
            let (aa, bb, cc) = inverse(alpha, beta);

            let sum = aa + bb + cc;
            assert!(
                sum.abs() <= TOLERANCE,
                "back-transform sum not zero: {},{},{}, sum={}",
                aa,
                bb,
                cc,
                sum
            );
        }
    }
}
