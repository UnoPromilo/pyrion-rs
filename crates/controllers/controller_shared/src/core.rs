use crate::converters::{
    convert_to_current, convert_to_temperature, convert_to_voltage, ConfigValues,
};
use crate::io::{RawInverterValues, RawSnapshot};
use core::sync::atomic::Ordering;
use foc::snapshot::ControlSnapshot;
use units::{ElectricCurrent, ElectricPotential, ThermodynamicTemperature};

pub fn control_step(raw_snapshot: &Option<RawSnapshot>) -> Option<RawInverterValues> {
    let _snapshot = match raw_snapshot {
        Some(values) => {
            let default_config: ConfigValues = ConfigValues::default();
            let i_u = convert_to_current(values.i_u, values.v_ref, &default_config);
            let i_v = convert_to_current(values.i_v, values.v_ref, &default_config);
            let i_w = convert_to_current(values.i_w, values.v_ref, &default_config);
            let bus_voltage = convert_to_voltage(values.v_bus as i32, values.v_ref);
            let cpu_temp = convert_to_temperature(values.temp_cpu, values.v_ref);

            store_in_state(i_u, i_v, i_w, bus_voltage, cpu_temp);

            Some(ControlSnapshot {
                phase_current: [i_u, i_v, i_w],
                bus_voltage,
            })
        }
        None => None,
    };

    Some(RawInverterValues {
        u: 1400,
        v: 0,
        w: 0,
    })
}

pub fn store_in_state(
    i_u: ElectricCurrent,
    i_v: ElectricCurrent,
    i_w: ElectricCurrent,
    v_bus: ElectricPotential,
    cpu_temp: ThermodynamicTemperature,
) {
    let state = crate::state::state();

    state.cpu_temp.store(cpu_temp.value, Ordering::Relaxed);
    state.i_u.store(i_u.value, Ordering::Relaxed);
    state.i_v.store(i_v.value, Ordering::Relaxed);
    state.i_w.store(i_w.value, Ordering::Relaxed);
    state.v_bus.store(v_bus.value, Ordering::Relaxed);
}
