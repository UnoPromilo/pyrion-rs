use crate::converters::{ConfigValues, convert_to_current, convert_to_voltage};
use crate::io::{RawInverterValues, RawSnapshot};
use foc::snapshot::ControlSnapshot;

pub fn control_step(raw_snapshot: &Option<RawSnapshot>) -> Option<RawInverterValues> {
    let _snapshot = match raw_snapshot {
        Some(values) => {
            let default_config: ConfigValues = ConfigValues::default();
            let i_u = convert_to_current(values.i_u, values.v_ref, &default_config);
            let i_v = convert_to_current(values.i_v, values.v_ref, &default_config);
            let i_w = convert_to_current(values.i_w, values.v_ref, &default_config);
            let bus_voltage = convert_to_voltage(values.v_bus as i32, values.v_ref);
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
