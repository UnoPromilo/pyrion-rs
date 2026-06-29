use units::si::electric_potential::millivolt;
use units::si::electrical_resistance::milliohm;
use units::si::thermodynamic_temperature::degree_celsius;
use units::{ElectricCurrent, ElectricPotential, ElectricalResistance, ThermodynamicTemperature};

pub fn convert_to_current(
    sample: u16,
    vrefint_sample: u16,
    config: &ConfigValues,
) -> ElectricCurrent {
    let zeroed_sample = sample as i32 - config.current_zero_offset;
    let voltage = convert_to_voltage(zeroed_sample, vrefint_sample);
    voltage / config.current_gain / config.shunt_resistance
}

pub fn convert_to_voltage(sample: i32, vrefint_sample: u16) -> ElectricPotential {
    let mv = convert_to_millivolts(sample, vrefint_sample);
    ElectricPotential::new::<millivolt>(mv as f32)
}

pub fn convert_to_temperature(sample: u16, vrefint_sample: u16) -> ThermodynamicTemperature {
    // Check the RM0440 and DS12288 for more details on temperature sensor and how to use calibration values
    const V30: i32 = 760; // mV
    const AVG_SLOPE: f32 = 2.5; // mV/C
    let mv = convert_to_millivolts(sample as i32, vrefint_sample);
    let temp_c = (mv - V30) as f32 / AVG_SLOPE + 30.0;
    ThermodynamicTemperature::new::<degree_celsius>(temp_c)
}

fn convert_to_millivolts(sample: i32, vrefint_sample: u16) -> i32 {
    const VREFINT_MV: i32 = 1210; // mV
    sample * VREFINT_MV / (vrefint_sample as i32)
}

#[derive(Debug)]
pub struct ConfigValues {
    // Current sensing
    pub shunt_resistance: ElectricalResistance,
    pub current_zero_offset: i32,
    pub current_gain: f32,

    // Voltage sensing
    pub v_bus_scale_ratio: f32,
}

// TODO remove default values, they should be taken from the config file
impl Default for ConfigValues {
    fn default() -> Self {
        Self {
            shunt_resistance: ElectricalResistance::new::<milliohm>(5.0),
            v_bus_scale_ratio: (39.0 + 2.0) / 2.0,
            current_gain: 20.0,
            current_zero_offset: 2048,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use units::si::electric_current::milliampere;
    use units::si::electrical_resistance::milliohm;
    const VREFINT: u16 = 1550;

    #[test]
    fn test_convert_to_current() {
        struct TestCase {
            sample: u16,
            vrefint: u16,
            expected: f32,
            config: ConfigValues,
        }

        let test_cases = [
            TestCase {
                sample: 2048,
                vrefint: VREFINT,
                expected: 0.0,
                config: default_config(),
            },
            TestCase {
                sample: 2486,
                vrefint: VREFINT,
                expected: 170.5,
                config: default_config(),
            },
            TestCase {
                sample: 1622,
                vrefint: VREFINT,
                expected: -166.0,
                config: default_config(),
            },
            TestCase {
                sample: 1622,
                vrefint: VREFINT,
                expected: -3320.0,
                config: {
                    let mut config = default_config();
                    config.shunt_resistance = ElectricalResistance::new::<milliohm>(10.0);
                    config.current_gain = 10.0;
                    config
                },
            },
        ];

        for test_case in test_cases {
            let result = convert_to_current(test_case.sample, test_case.vrefint, &test_case.config);
            let raw_result = result.get::<milliampere>();
            assert_eq!(
                raw_result,
                test_case.expected,
                "Expected {}mA, got {}mA, error: {}mA",
                test_case.expected,
                raw_result,
                raw_result - test_case.expected
            )
        }
    }

    #[test]
    fn test_convert_to_voltage() {
        struct TestCase {
            sample: i32,
            vrefint: u16,
            expected: f32,
        }

        let test_cases = [
            TestCase {
                sample: 0,
                vrefint: VREFINT,
                expected: 0.0,
            },
            TestCase {
                sample: 4096,
                vrefint: VREFINT,
                expected: 3197.0,
            },
        ];

        for test_case in test_cases {
            let result = convert_to_voltage(test_case.sample, test_case.vrefint);
            let raw_result = result.get::<millivolt>();
            assert_eq!(
                raw_result,
                test_case.expected,
                "Expected {}mV, got {}mV, error: {}mV",
                test_case.expected,
                raw_result,
                raw_result - test_case.expected
            )
        }
    }
    fn default_config() -> ConfigValues {
        ConfigValues {
            shunt_resistance: ElectricalResistance::new::<milliohm>(100.0),
            current_gain: 20.0,
            current_zero_offset: 2048,
            v_bus_scale_ratio: (39.0 + 2.0) / 2.0,
        }
    }
}
