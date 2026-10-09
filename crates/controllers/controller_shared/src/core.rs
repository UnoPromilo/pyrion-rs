use crate::converters::{
    ConfigValues, convert_to_current, convert_to_temperature, convert_to_voltage,
};
use crate::io::{ControlFault, ControlOutput, RawInverterValues, RawSnapshot};
use crate::strategy::ControlStrategy;
use core::sync::atomic::Ordering;
use foc::snapshot::{AngleSnapshot, FocInput};
use units::si::angle::radian;
use units::{
    Angle, ElectricCurrent, ElectricPotential, IntoRawDutyCycle, ThermodynamicTemperature,
};

pub fn control_step(
    raw_snapshot: &RawSnapshot,
    control_strategy: &mut ControlStrategy,
) -> ControlOutput {
    if raw_snapshot.v_ref == 0 {
        return ControlOutput::Fault(ControlFault::InvalidMeasurement);
    }

    let default_config: ConfigValues = ConfigValues::default();
    let u = convert_to_current(raw_snapshot.i_u, raw_snapshot.v_ref, &default_config);
    let v = convert_to_current(raw_snapshot.i_v, raw_snapshot.v_ref, &default_config);
    let w = convert_to_current(raw_snapshot.i_w, raw_snapshot.v_ref, &default_config);
    let v_bus = convert_to_voltage(raw_snapshot.v_bus as i32, raw_snapshot.v_ref)
        * default_config.v_bus_scale_ratio;
    let cpu_temp = convert_to_temperature(raw_snapshot.temp_cpu, raw_snapshot.v_ref);

    store_in_state(u, v, w, v_bus, cpu_temp);

    if !u.value.is_finite()
        || !v.value.is_finite()
        || !w.value.is_finite()
        || !v_bus.value.is_finite()
        || v_bus.value <= 0.0
        || !cpu_temp.value.is_finite()
    {
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
                v_bus,
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

pub fn store_in_state(
    i_u: ElectricCurrent,
    i_v: ElectricCurrent,
    i_w: ElectricCurrent,
    v_bus: ElectricPotential,
    cpu_temp: ThermodynamicTemperature,
) {
    let state = crate::state::state();

    state.cpu_temp.store(cpu_temp, Ordering::Relaxed);
    state.i_u.store(i_u, Ordering::Relaxed);
    state.i_v.store(i_v, Ordering::Relaxed);
    state.i_w.store(i_w, Ordering::Relaxed);
    state.v_bus.store(v_bus, Ordering::Relaxed);
}

#[cfg(test)]
mod tests {
    use super::*;
    use foc::state::FocState;
    use units::si::ratio::ratio;
    use units::{F32UnitType, Ratio};

    fn valid_snapshot() -> RawSnapshot {
        RawSnapshot {
            i_u: 2048,
            i_v: 2048,
            i_w: 2048,
            v_u: 0,
            v_v: 0,
            v_w: 0,
            v_ref: 1550,
            v_bus: 1000,
            temp_cpu: 1000,
            temp_motor: 0,
            temp_driver: 0,
            analog_input: 0,
            max_duty: 1000,
            angle: 0,
        }
    }

    #[test]
    fn disabled_strategy_requests_safe_output() {
        let mut strategy = ControlStrategy::Disabled;

        assert_eq!(
            control_step(&valid_snapshot(), &mut strategy),
            ControlOutput::Safe
        );
    }

    #[test]
    fn zero_voltage_reference_is_an_explicit_fault() {
        let mut snapshot = valid_snapshot();
        snapshot.v_ref = 0;
        let mut strategy = ControlStrategy::Disabled;

        assert_eq!(
            control_step(&snapshot, &mut strategy),
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
            ElectricPotential::from_f32(10.0),
            ElectricPotential::from_f32(-10.0),
        );
        state.q_requested = ElectricCurrent::from_f32(f32::NAN);
        let mut strategy = ControlStrategy::Foc(state);

        assert_eq!(
            control_step(&valid_snapshot(), &mut strategy),
            ControlOutput::Fault(ControlFault::InvalidControllerOutput)
        );
    }

    #[test]
    fn zero_bus_voltage_is_an_explicit_fault() {
        let mut snapshot = valid_snapshot();
        snapshot.v_bus = 0;
        let mut strategy = ControlStrategy::Disabled;

        assert_eq!(
            control_step(&snapshot, &mut strategy),
            ControlOutput::Fault(ControlFault::InvalidMeasurement)
        );
    }
}
