use crate::converters::{
    BoardSensorScales, convert_to_current, convert_to_voltage, current_per_adc_count,
};
use crate::io::{ControlFault, ControlOutput, CurrentZeroOffsets, RawInverterValues, RawSnapshot};
use crate::strategy::ControlStrategy;
use core::num::NonZeroU16;
use core::sync::atomic::Ordering;
use foc::snapshot::{AngleSnapshot, FocInput};
use units::si::angle::radian;
use units::{Angle, ElectricCurrent, IntoRawDutyCycle};

pub fn store_bus_voltage(
    sample: u16,
    vref: NonZeroU16,
    vref_cal: NonZeroU16,
    scales: &BoardSensorScales,
) {
    let v_bus =
        convert_to_voltage(sample as i32, vref.get(), vref_cal.get()) * scales.v_bus_scale_ratio;
    crate::state::state().v_bus.store(v_bus, Ordering::Relaxed);
}

pub fn control_step(
    raw_snapshot: &RawSnapshot,
    control_strategy: &mut ControlStrategy,
    current_zero: &CurrentZeroOffsets,
    vrefint_cal: u16,
    scales: &BoardSensorScales,
) -> ControlOutput {
    if raw_snapshot.v_ref == 0 || !(1..=4095).contains(&vrefint_cal) {
        return ControlOutput::Fault(ControlFault::InvalidMeasurement);
    }

    let [u, v, w] = phase_currents(raw_snapshot, current_zero, vrefint_cal, scales);
    store_in_state(u, v, w);

    if !u.value.is_finite() || !v.value.is_finite() || !w.value.is_finite() {
        return ControlOutput::Fault(ControlFault::InvalidMeasurement);
    }

    match control_strategy {
        ControlStrategy::Disabled => ControlOutput::Safe,
        ControlStrategy::Foc(state) => {
            let input = FocInput {
                // TODO take real angle values
                angle: AngleSnapshot {
                    value: Angle::new::<radian>(0.0),
                    sin: 0.0,
                    cos: 1.0,
                },
                u,
                v,
                w,
            };
            let output = foc::core::foc_step(input, state);
            if !output.u.value.is_finite()
                || !output.v.value.is_finite()
                || !output.w.value.is_finite()
            {
                return ControlOutput::Fault(ControlFault::InvalidControllerOutput);
            }

            ControlOutput::Drive(RawInverterValues {
                u: output.u.into_raw_duty_cycle(raw_snapshot.max_duty),
                v: output.v.into_raw_duty_cycle(raw_snapshot.max_duty),
                w: output.w.into_raw_duty_cycle(raw_snapshot.max_duty),
            })
        }
    }
}

fn phase_currents(
    raw: &RawSnapshot,
    zero: &CurrentZeroOffsets,
    vrefint_cal: u16,
    scales: &BoardSensorScales,
) -> [ElectricCurrent; 3] {
    let per_count = current_per_adc_count(raw.v_ref, vrefint_cal, scales);
    [
        convert_to_current(raw.i_u, zero.u, per_count),
        convert_to_current(raw.i_v, zero.v, per_count),
        convert_to_current(raw.i_w, zero.w, per_count),
    ]
}

pub fn store_in_state(i_u: ElectricCurrent, i_v: ElectricCurrent, i_w: ElectricCurrent) {
    let state = crate::state::state();

    state.i_u.store(i_u, Ordering::Relaxed);
    state.i_v.store(i_v, Ordering::Relaxed);
    state.i_w.store(i_w, Ordering::Relaxed);
}

#[cfg(test)]
mod tests {
    use super::*;
    use foc::state::FocState;
    use units::si::electrical_resistance::milliohm;
    use units::si::ratio::ratio;
    use units::{ElectricalResistance, F32UnitType, Ratio};
    const VREFINT_CAL: u16 = 1652;

    fn valid_snapshot() -> RawSnapshot {
        RawSnapshot {
            i_u: 2048,
            i_v: 2048,
            i_w: 2048,
            v_u: 0,
            v_v: 0,
            v_w: 0,
            v_ref: 1550,
            max_duty: 1000,
            angle: 0,
        }
    }

    fn zero_offsets() -> CurrentZeroOffsets {
        CurrentZeroOffsets {
            u: 2048,
            v: 2048,
            w: 2048,
        }
    }

    fn test_scales() -> BoardSensorScales {
        BoardSensorScales {
            shunt_resistance: ElectricalResistance::new::<milliohm>(0.5),
            current_gain: 20.0,
            v_bus_scale_ratio: 2.0,
        }
    }

    #[test]
    fn each_phase_uses_its_own_zero_count() {
        let mut raw = valid_snapshot();
        raw.i_u = 2073;
        raw.i_v = 2060;
        raw.i_w = 2030;
        let zero = CurrentZeroOffsets {
            u: 2073,
            v: 2060,
            w: 2030,
        };
        let config = test_scales();

        let [u, v, w] = phase_currents(&raw, &zero, VREFINT_CAL, &config);
        assert_eq!([u.value, v.value, w.value], [0.0; 3]);

        raw.i_v += 1;
        let [u, v, w] = phase_currents(&raw, &zero, VREFINT_CAL, &config);
        assert_eq!(u.value, 0.0);
        assert!(v.value > 0.0);
        assert_eq!(w.value, 0.0);
    }

    #[test]
    fn phase_currents_use_configured_gain() {
        let mut raw = valid_snapshot();
        raw.i_u += 1;
        let config = test_scales();
        let initial = phase_currents(&raw, &zero_offsets(), VREFINT_CAL, &config)[0];
        let doubled_gain = BoardSensorScales {
            current_gain: config.current_gain * 2.0,
            ..config
        };
        let reduced = phase_currents(&raw, &zero_offsets(), VREFINT_CAL, &doubled_gain)[0];

        assert!((initial.value - reduced.value * 2.0).abs() < 0.0001);
    }

    #[test]
    fn one_reference_scale_applies_to_signed_phase_deltas() {
        let mut raw = valid_snapshot();
        raw.i_u = 2050;
        raw.i_v = 2046;
        raw.i_w = 2049;

        let [u, v, w] = phase_currents(&raw, &zero_offsets(), VREFINT_CAL, &test_scales());

        assert!(u.value > 0.0);
        assert!((u.value + v.value).abs() < 0.000001);
        assert!((u.value - 2.0 * w.value).abs() < 0.000001);
    }

    #[test]
    fn disabled_strategy_requests_safe_output() {
        let mut strategy = ControlStrategy::Disabled;

        assert_eq!(
            control_step(
                &valid_snapshot(),
                &mut strategy,
                &zero_offsets(),
                VREFINT_CAL,
                &test_scales(),
            ),
            ControlOutput::Safe
        );
    }

    #[test]
    fn zero_voltage_reference_is_an_explicit_fault() {
        let mut snapshot = valid_snapshot();
        snapshot.v_ref = 0;
        let mut strategy = ControlStrategy::Disabled;

        assert_eq!(
            control_step(
                &snapshot,
                &mut strategy,
                &zero_offsets(),
                VREFINT_CAL,
                &test_scales()
            ),
            ControlOutput::Fault(ControlFault::InvalidMeasurement)
        );
    }

    #[test]
    fn non_finite_controller_output_is_an_explicit_fault() {
        let mut state = FocState::new(
            Ratio::new::<ratio>(1.0),
            Ratio::new::<ratio>(0.0),
            10.0,
            -10.0,
            Ratio::new::<ratio>(0.5),
            Ratio::new::<ratio>(-0.5),
        );
        state.q_requested = ElectricCurrent::from_f32(f32::NAN);
        let mut strategy = ControlStrategy::Foc(state);

        assert_eq!(
            control_step(
                &valid_snapshot(),
                &mut strategy,
                &zero_offsets(),
                VREFINT_CAL,
                &test_scales(),
            ),
            ControlOutput::Fault(ControlFault::InvalidControllerOutput)
        );
    }

    #[test]
    fn motor_control_does_not_use_bus_voltage_telemetry() {
        store_bus_voltage(
            0,
            NonZeroU16::new(1550).unwrap(),
            NonZeroU16::new(VREFINT_CAL).unwrap(),
            &test_scales(),
        );
        let state = FocState::new(
            Ratio::new::<ratio>(0.0),
            Ratio::new::<ratio>(0.0),
            0.5,
            -0.5,
            Ratio::new::<ratio>(0.5),
            Ratio::new::<ratio>(-0.5),
        );
        let mut strategy = ControlStrategy::Foc(state);

        let at_zero_bus = control_step(
            &valid_snapshot(),
            &mut strategy,
            &zero_offsets(),
            VREFINT_CAL,
            &test_scales(),
        );
        assert_eq!(
            at_zero_bus,
            ControlOutput::Drive(RawInverterValues {
                u: 500,
                v: 500,
                w: 500
            })
        );

        store_bus_voltage(
            1000,
            NonZeroU16::new(1550).unwrap(),
            NonZeroU16::new(VREFINT_CAL).unwrap(),
            &test_scales(),
        );
        let initial = crate::state::state().v_bus.load(Ordering::Relaxed).value;
        assert!((initial - 1.5616).abs() < 0.001);
        store_bus_voltage(
            1000,
            NonZeroU16::new(1550).unwrap(),
            NonZeroU16::new(VREFINT_CAL).unwrap(),
            &BoardSensorScales {
                v_bus_scale_ratio: 4.0,
                ..test_scales()
            },
        );
        let scaled = crate::state::state().v_bus.load(Ordering::Relaxed).value;
        assert!((scaled - 2.0 * initial).abs() < 0.0001);
        assert_eq!(
            control_step(
                &valid_snapshot(),
                &mut strategy,
                &zero_offsets(),
                VREFINT_CAL,
                &test_scales(),
            ),
            at_zero_bus
        );
    }
}
