use crate::io::{RawInverterValues, RawSnapshot};

pub fn control_step(_raw_values: &Option<RawSnapshot>) -> Option<RawInverterValues> {
    Some(RawInverterValues { u: 1400, v: 0, w: 0 })
}

#[allow(dead_code)]
fn convert_to_millivolts(sample: u16, vrefint_sample: u16) -> u16 {
    const VREFINT_MV: u32 = 1210; // mV
    (u32::from(sample) * VREFINT_MV / u32::from(vrefint_sample)) as u16
}
