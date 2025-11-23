use units::si::Quantity;
use units::{DutyCycle, ElectricPotential, Ratio};

pub const ONE_OVER_SQRT3: f32 = 0.577_350_26_f32;
pub const TWO_OVER_SQRT3: f32 = ONE_OVER_SQRT3 * 2f32;

pub fn alternate_reverse_space_vector_modulation(
    alpha: ElectricPotential,
    beta: ElectricPotential,
    v_bus: ElectricPotential,
) -> (DutyCycle, DutyCycle, DutyCycle) {
    let sector = get_sector(alpha, beta);
    let (t_a, t_b) = calculate_vector_times(alpha, beta, v_bus, sector);
    calculate_duty_times(t_a, t_b, sector)
}

fn get_sector(alpha: ElectricPotential, beta: ElectricPotential) -> Sector {
    let a = alpha.value;
    let b = beta.value;

    match (a >= 0.0, b >= 0.0) {
        // Quadrant I
        (true, true) => {
            if a >= ONE_OVER_SQRT3 * b {
                Sector::First
            } else {
                Sector::Second
            }
        }

        // Quadrant II
        (false, true) => {
            if a >= -ONE_OVER_SQRT3 * b {
                Sector::Second
            } else {
                Sector::Third
            }
        }

        // Quadrant III
        (false, false) => {
            if a >= ONE_OVER_SQRT3 * b {
                Sector::Fifth
            } else {
                Sector::Fourth
            }
        }

        // Quadrant IV
        (true, false) => {
            if a >= -ONE_OVER_SQRT3 * b {
                Sector::Sixth
            } else {
                Sector::Fifth
            }
        }
    }
}

fn calculate_vector_times(
    alpha: ElectricPotential,
    beta: ElectricPotential,
    v_bus: ElectricPotential,
    sector: Sector,
) -> (Ratio, Ratio) {
    let alpha_normalized = alpha / v_bus;
    let beta_normalized = beta / v_bus;
    match sector {
        Sector::First => (
            alpha_normalized - ONE_OVER_SQRT3 * beta_normalized,
            TWO_OVER_SQRT3 * beta_normalized,
        ),
        Sector::Second => (
            alpha_normalized + ONE_OVER_SQRT3 * beta_normalized,
            -alpha_normalized + ONE_OVER_SQRT3 * beta_normalized,
        ),
        Sector::Third => (
            TWO_OVER_SQRT3 * beta_normalized,
            -alpha_normalized - ONE_OVER_SQRT3 * beta_normalized,
        ),
        Sector::Fourth => (
            -alpha_normalized + ONE_OVER_SQRT3 * beta_normalized,
            -TWO_OVER_SQRT3 * beta_normalized,
        ),
        Sector::Fifth => (
            -alpha_normalized - ONE_OVER_SQRT3 * beta_normalized,
            alpha_normalized - ONE_OVER_SQRT3 * beta_normalized,
        ),
        Sector::Sixth => (
            -TWO_OVER_SQRT3 * beta_normalized,
            alpha_normalized + ONE_OVER_SQRT3 * beta_normalized,
        ),
    }
}

fn calculate_duty_times(
    t_a: Ratio,
    t_b: Ratio,
    sector: Sector,
) -> (DutyCycle, DutyCycle, DutyCycle) {
    let one = Quantity::from(1.0);

    match sector {
        Sector::First => {
            // base = (1 + t1 + t2)/2
            let u = (one + t_a + t_b) / 2.0;
            let v = u - t_a;
            let w = v - t_b;
            (u, v, w)
        }
        Sector::Second => {
            let v = (one + t_a + t_b) / 2.0;
            let u = v - t_b;
            let w = u - t_a;
            (u, v, w)
        }
        Sector::Third => {
            let v = (one + t_a + t_b) / 2.0;
            let w = v - t_a;
            let u = w - t_b;
            (u, v, w)
        }
        Sector::Fourth => {
            let w = (one + t_a + t_b) / 2.0;
            let v = w - t_b;
            let u = v - t_a;
            (u, v, w)
        }
        Sector::Fifth => {
            let w = (one + t_a + t_b) / 2.0;
            let u = w - t_a;
            let v = u - t_b;
            (u, v, w)
        }
        Sector::Sixth => {
            let u = (one + t_a + t_b) / 2.0;
            let w = u - t_b;
            let v = w - t_a;
            (u, v, w)
        }
    }
}
#[derive(Debug, PartialEq, Clone, Copy)]
enum Sector {
    First,
    Second,
    Third,
    Fourth,
    Fifth,
    Sixth,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clarke_transformation::full_clarke_transformation;
    use units::{ElectricCurrent, F32UnitType};

    const SQRT3_OVER_TWO: f32 = 0.866_025_4_f32;
    const EPS: f32 = 1e-4;

    fn expected_sector(angle_deg: f32) -> Sector {
        let a = (angle_deg % 360.0 + 360.0) % 360.0;

        match a {
            a if a < 60.0 => Sector::First,
            a if a < 120.0 => Sector::Second,
            a if a < 180.0 => Sector::Third,
            a if a < 240.0 => Sector::Fourth,
            a if a < 300.001 => Sector::Fifth,
            _ => Sector::Sixth,
        }
    }

    #[test]
    fn test_sector_classifier_against_truth_table() {
        for deg in 0..36000 {
            let angle = (deg as f32) / 100.0;

            // test coordinates
            let rad = angle.to_radians();
            let alpha = rad.cos();
            let beta = rad.sin();

            let sector = get_sector(
                ElectricPotential::from_f32(alpha),
                ElectricPotential::from_f32(beta),
            );
            let expected = expected_sector(angle);

            assert_eq!(
                sector, expected,
                "Wrong sector for an angle {}°: got {:?}, expected {:?}",
                angle, sector, expected
            );
        }
    }

    #[test]
    fn svm_round_trip_alpha_beta() {
        let v_bus = ElectricPotential::from_f32(1.0);

        for deg in 0..360 {
            let theta = (deg as f32).to_radians();
            let alpha_norm = SQRT3_OVER_TWO * theta.cos();
            let beta_norm = SQRT3_OVER_TWO * theta.sin();

            let alpha = ElectricPotential::from_f32(alpha_norm);
            let beta = ElectricPotential::from_f32(beta_norm);

            let (u, v, w) = alternate_reverse_space_vector_modulation(alpha, beta, v_bus);
            let (alpha_rec, beta_rec) = reconstruct_alpha_beta(u, v, w);

            assert!(
                approx_eq(alpha_rec, alpha_norm, EPS),
                "alpha mismatch at {}°: got {}, expected {}",
                deg,
                alpha_rec,
                alpha_norm
            );
            assert!(
                approx_eq(beta_rec, beta_norm, EPS),
                "beta mismatch at {}°: got {}, expected {}",
                deg,
                beta_rec,
                beta_norm
            );
        }
    }

    #[test]
    fn make_sure_output_is_always_between_one_and_zero() {
        for deg in 0..3600 {
            let angle = (deg as f32) / 10.0;

            let rad = angle.to_radians();
            let m = SQRT3_OVER_TWO - EPS;
            let alpha = ElectricPotential::from_f32(rad.cos() * m);
            let beta = ElectricPotential::from_f32(rad.sin() * m);
            let v_bus = ElectricPotential::from_f32(1.0);

            let (u, v, w) = alternate_reverse_space_vector_modulation(alpha, beta, v_bus);
            let (u, v, w) = (u.value, v.value, w.value);
            assert!(
                (0.0..=1.0).contains(&u),
                "u has invalid value {} at {}°",
                u,
                angle
            );
            assert!(
                (0.0..=1.0).contains(&v),
                "v has invalid value {} at {}°",
                v,
                angle
            );
            assert!(
                (0.0..=1.0).contains(&w),
                "w has invalid value {} at {}°",
                w,
                angle
            )
        }
    }

    fn reconstruct_alpha_beta(u: DutyCycle, v: DutyCycle, w: DutyCycle) -> (f32, f32) {
        let (u, v, w) = (u.value, v.value, w.value);
        let common = (u + v + w) / 3.0;
        // We are operating on potential not on current, but the math is unit agnostic
        let v_u = ElectricCurrent::from_f32(u - common);
        let v_v = ElectricCurrent::from_f32(v - common);
        let v_w = ElectricCurrent::from_f32(w - common);
        let (alpha, beta) = full_clarke_transformation(v_u, v_v, v_w);
        (alpha.value * 3.0 / 2.0, beta.value * 3.0 / 2.0)
    }

    fn approx_eq(a: f32, b: f32, eps: f32) -> bool {
        (a - b).abs() <= eps
    }
}
