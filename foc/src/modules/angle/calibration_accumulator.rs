use fixed::types::{I1F15, I3F29, I16F48};
use shared::functions::atan::atan2;
use shared::units::Angle;
use shared::units::angle::{Electrical, Mechanical};

pub struct CalibrationAccumulator<const P_MAX: usize> {
    sums_cw: [(I16F48, I16F48); P_MAX],
    sums_ccw: [(I16F48, I16F48); P_MAX],
    count: usize,
}

#[derive(Debug)]
pub struct Result {
    pub pole_pairs: i16,
    pub offset: Angle<Electrical>,
    pub coherence: I1F15,
}

impl<const P_MAX: usize> CalibrationAccumulator<P_MAX> {
    const ZERO: I16F48 = I16F48::lit("0");
    pub fn new() -> Self {
        Self {
            sums_cw: [(Self::ZERO, Self::ZERO); P_MAX],
            sums_ccw: [(Self::ZERO, Self::ZERO); P_MAX],
            count: 0,
        }
    }

    pub fn add_sample(
        &mut self,
        angle_electrical: &Angle<Electrical>,
        angle_mechanical: &Angle<Mechanical>,
    ) {
        const ZERO_ANGLE: Angle<Electrical> = Angle::zero();
        self.count += 1;

        for pairs in 1..=P_MAX {
            let phase_error_clockwise = angle_electrical.overflowing_sub(&Angle::from_mechanical(
                angle_mechanical,
                &ZERO_ANGLE,
                pairs as i16,
            ));

            let phase_error_counter_clockwise = angle_electrical.overflowing_sub(
                &Angle::from_mechanical(angle_mechanical, &ZERO_ANGLE, -(pairs as i16)),
            );

            let (ref mut sin_cw, ref mut cos_cw) = self.sums_cw[pairs - 1];
            let (ref mut sin_ccw, ref mut cos_ccw) = self.sums_ccw[pairs - 1];

            *sin_cw += phase_error_clockwise.sin().to_num::<I16F48>();
            *cos_cw += phase_error_clockwise.cos().to_num::<I16F48>();

            *sin_ccw += phase_error_counter_clockwise.sin().to_num::<I16F48>();
            *cos_ccw += phase_error_counter_clockwise.cos().to_num::<I16F48>();
        }
    }

    pub fn finalize(&self) -> Result {
        let mut best_strength = I16F48::lit("0");
        let mut best_pp: i16 = 0;
        let mut best_angle_rad: I3F29 = I3F29::lit("0");

        // If the assumed pole-pair count is correct, all sampled sin/cos values
        // (ignoring noise) should be consistent and align on the unit circle.
        // Averaging them should still yield a vector with magnitude ≈ 1,
        // satisfying the Pythagorean identity.
        // If the assumption is wrong, the averages cancel out and the magnitude drops.
        // This lets us identify the true pole-pair count by picking the index
        // with the strongest (closest to 1) magnitude
        for (i, (&(sin_cw, cos_cw), &(sin_ccw, cos_ccw))) in
            self.sums_cw.iter().zip(self.sums_ccw.iter()).enumerate()
        {
            let avg_sin_cw = sin_cw / I16F48::from_num(self.count as i16);
            let avg_cos_cw = cos_cw / I16F48::from_num(self.count as i16);
            let mag_cw = avg_sin_cw * avg_sin_cw + avg_cos_cw * avg_cos_cw;
            let mag_cw = mag_cw.sqrt();
            if mag_cw > best_strength {
                best_strength = mag_cw;
                best_pp = i as i16 + 1;
                best_angle_rad = atan2(avg_sin_cw.to_num(), avg_cos_cw.to_num());
            }

            let avg_sin_ccw = sin_ccw / I16F48::from_num(self.count as i16);
            let avg_cos_ccw = cos_ccw / I16F48::from_num(self.count as i16);
            let mag_ccw = avg_sin_ccw * avg_sin_ccw + avg_cos_ccw * avg_cos_ccw;
            if mag_ccw > best_strength {
                best_strength = mag_ccw;
                best_pp = -(i as i16 + 1);
                best_angle_rad = atan2(avg_sin_cw.to_num(), avg_cos_cw.to_num());
            }
        }

        let coherence = if self.count > 0 {
            (best_strength / I16F48::from_num(self.count)).to_num::<I1F15>()
        } else {
            I1F15::from_num(0)
        };

        if best_angle_rad < 0 {
            best_angle_rad = best_angle_rad
                + fixed::consts::PI.to_num::<I3F29>()
                + fixed::consts::PI.to_num::<I3F29>();
        }

        Result {
            pole_pairs: best_pp,
            offset: Angle::from_rad(best_angle_rad.to_num()),
            coherence,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;
    use shared::units::Angle;
    use shared::units::angle::{Electrical, Mechanical};

    fn run_test_case(true_poles: i16, noise_raw: u16) {
        const P_MAX: usize = 12;
        let mut acc = CalibrationAccumulator::<P_MAX>::new();
        let steps = 10;
        for i in 0..steps {
            let mut mech_raw: u16 = (i * 65535 / 360) as u16;

            if noise_raw > 0 {
                let delta: i32 = rand::rng().random_range(-(noise_raw as i32)..=(noise_raw as i32));
                mech_raw = mech_raw.wrapping_add(delta as u16);
            }

            let mechanical = Angle::<Mechanical>::from_raw(mech_raw);
            let electrical =
                Angle::<Electrical>::from_mechanical(&mechanical, &Angle::zero(), true_poles);
            acc.add_sample(&electrical, &mechanical);
        }

        let result = acc.finalize();
        assert_eq!(
            result.pole_pairs, true_poles,
            "Failed for true_poles = {}",
            true_poles
        );
    }

    #[test]
    fn test_multiple_pole_cases() {
        // test positive pole pairs
        for poles in 1..=12 {
            run_test_case(poles, 0);
        }

        // test negative pole pairs
        for poles in -12..=-1 {
            run_test_case(poles, 0);
        }
    }

    #[test]
    fn test_with_noise() {
        // ±1° noise in raw units: u16::MAX / 360 ≈ 182
        let noise_raw = u16::MAX / 360;
        run_test_case(7, noise_raw);
        run_test_case(-5, noise_raw);
    }
}
