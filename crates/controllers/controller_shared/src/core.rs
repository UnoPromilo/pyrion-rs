use crate::command::{ControlCommand, ControlCommandChannel};
use crate::converters::{
    ConfigValues, convert_to_current, convert_to_temperature, convert_to_voltage,
};
use crate::io::{RawInverterValues, RawSnapshot};
use crate::strategy::ControlStrategy;
use core::sync::atomic::Ordering;
use foc::snapshot::{AngleSnapshot, FocInput};
use units::si::angle::radian;
use units::{
    Angle, ElectricCurrent, ElectricPotential, IntoRawDutyCycle, ThermodynamicTemperature,
};

pub fn update_strategy(
    command_channel: &ControlCommandChannel,
    current_strategy: ControlStrategy,
) -> ControlStrategy {
    let command = command_channel.try_receive().ok();
    match command {
        None => current_strategy,
        Some(ControlCommand::DisableMotor) => ControlStrategy::Disabled,
    }
}

pub fn control_step(
    raw_snapshot: &Option<RawSnapshot>,
    control_strategy: &mut ControlStrategy,
) -> Option<RawInverterValues> {
    match raw_snapshot {
        Some(values) => {
            let default_config: ConfigValues = ConfigValues::default();
            let u = convert_to_current(values.i_u, values.v_ref, &default_config);
            let v = convert_to_current(values.i_v, values.v_ref, &default_config);
            let w = convert_to_current(values.i_w, values.v_ref, &default_config);
            let v_bus = convert_to_voltage(values.v_bus as i32, values.v_ref);
            let cpu_temp = convert_to_temperature(values.temp_cpu, values.v_ref);
            store_in_state(u, v, w, v_bus, cpu_temp);

            match control_strategy {
                ControlStrategy::Disabled => None,
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
                    Some(RawInverterValues {
                        u: output.u.into_raw_duty_cycle(values.max_duty),
                        v: output.v.into_raw_duty_cycle(values.max_duty),
                        w: output.w.into_raw_duty_cycle(values.max_duty),
                    })
                }
            }
        }
        None => None,
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
