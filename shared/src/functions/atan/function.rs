use crate::functions::atan::generated::ATAN_TABLE;
use fixed::types::{I32F96, I3F29};

const ZERO: I32F96 = I32F96::lit("0");
const ONE: I32F96 = I32F96::lit("1");

pub fn atan(val: I32F96) -> I3F29 {
    cordic_circular(ONE, val, ZERO, ZERO).2.to_num()
}

/// Compute the arc-tangent of `y/x` with quadrant correction.
pub fn atan2(y: I32F96, x: I32F96) -> I3F29 {
    if x == ZERO {
        return if y < ZERO {
            -fixed::consts::FRAC_PI_2.to_num::<I3F29>()
        } else {
            fixed::consts::FRAC_PI_2.to_num()
        };
    }

    if y == ZERO {
        return if x >= ZERO {
            ZERO.to_num()
        } else {
            fixed::consts::PI.to_num()
        }
    }

    match (x < ZERO, y < ZERO) {
        (false, false) => atan(y / x),
        (false, true) => -atan(-y / x),
        (true, false) => fixed::consts::PI.to_num::<I3F29>() - atan(y / -x),
        (true, true) => atan(y / x) - fixed::consts::PI.to_num::<I3F29>(),
    }
}

fn cordic_circular(
    mut x: I32F96,
    mut y: I32F96,
    mut z: I32F96,
    vec_mode: I32F96,
) -> (I32F96, I32F96, I32F96) {
    for i in 0..16 {
        if vec_mode >= ZERO && y < vec_mode || vec_mode < ZERO && z >= ZERO {
            let x1 = x - (y >> i);
            y = y + (x >> i);
            x = x1;
            z = z - ATAN_TABLE[i]
        } else {
            let x1 = x + (y >> i);
            y = y - (x >> i);
            x = x1;
            z = z + ATAN_TABLE[i]
        }
    }

    (x, y, z)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::units::angle::Electrical;
    use crate::units::Angle;
    use fixed::types::U16F16;
    const EPSILON: f64 = 0.001;

    fn approx_eq(a: I32F96, b: f64) -> bool {
        let diff = (a.to_num::<f64>() - b).abs();
        diff <= EPSILON
    }

    #[test]
    fn test_basic_quadrants() {
        // Quadrant I
        let y = I32F96::from_num(1.0);
        let x = I32F96::from_num(1.0);
        let angle = atan2(y, x);
        assert!(approx_eq(angle.to_num(), f64::atan2(1.0, 1.0)));

        // Quadrant II
        let y = I32F96::from_num(1.0);
        let x = I32F96::from_num(-1.0);
        let angle = atan2(y, x);
        assert!(approx_eq(angle.to_num(), f64::atan2(1.0, -1.0)));

        // Quadrant III
        let y = I32F96::from_num(-1.0);
        let x = I32F96::from_num(-1.0);
        let angle = atan2(y, x);
        assert!(approx_eq(angle.to_num(), f64::atan2(-1.0, -1.0)));

        // Quadrant IV
        let y = I32F96::from_num(-1.0);
        let x = I32F96::from_num(1.0);
        let angle = atan2(y, x);
        assert!(approx_eq(angle.to_num(), f64::atan2(-1.0, 1.0)));
    }

    #[test]
    fn test_axis_aligned() {
        // Positive x-axis
        let angle = atan2(I32F96::from_num(0.0), I32F96::from_num(1.0));
        assert!(approx_eq(angle.to_num(), f64::atan2(0.0, 1.0)));

        // Positive y-axis
        let angle = atan2(I32F96::from_num(1.0), I32F96::from_num(0.0));
        assert!(approx_eq(angle.to_num(), f64::atan2(1.0, 0.0)));

        // Negative x-axis
        let angle = atan2(I32F96::from_num(0.0), I32F96::from_num(-1.0));
        assert!(approx_eq(angle.to_num(), f64::atan2(0.0, -1.0)));

        // Negative y-axis
        let angle = atan2(I32F96::from_num(-1.0), I32F96::from_num(0.0));
        assert!(approx_eq(angle.to_num(), f64::atan2(-1.0, 0.0)));
    }

    #[test]
    fn test_random_values() {
        let test_cases = [
            (0.5, 0.5),
            (-0.5, 0.5),
            (0.5, -0.5),
            (-0.5, -0.5),
            (0.123, 0.456),
            (-0.789, 0.321),
        ];

        for &(y_f, x_f) in &test_cases {
            let y = I32F96::from_num(y_f);
            let x = I32F96::from_num(x_f);
            let angle = atan2(y, x);
            let expected = f64::atan2(y_f, x_f);
            assert!(approx_eq(angle.to_num(), expected), "y={}, x={}", y_f, x_f);
        }
    }

    #[test]
    fn test_10_deg() {
        let input:Angle<Electrical> = Angle::from_degrees(U16F16::const_from_int(10));
        let cos = input.cos();
        let sin = input.sin();
        let atan2_rad = atan2(sin.to_num(), cos.to_num());
        let output:Angle<Electrical> = Angle::from_rad(atan2_rad.to_num());
        assert_eq!(input, output);
    }
}
