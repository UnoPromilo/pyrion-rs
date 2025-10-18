use units::si::electric_potential::millivolt;
use units::si::thermodynamic_temperature::degree_celsius;
use units::{ElectricCurrent, ElectricPotential, ElectricalResistance, ThermodynamicTemperature};

pub fn convert_to_current(
    sample: u16,
    vrefint_sample: u16,
    config: &ConfigValues,
) -> ElectricCurrent {
    let zeroed_sample = sample as i32 - config.zero_point;
    let voltage = convert_to_voltage(zeroed_sample, vrefint_sample);
    voltage / config.gain / config.shunt_resistor
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
    const VREFINT_MV: i64 = 1210; // mV
    ((sample as i64) * VREFINT_MV / (vrefint_sample as i64)) as i32
}

#[derive(Debug)]
pub struct ConfigValues {
    pub shunt_resistor: ElectricalResistance,
    pub gain: f32,
    pub zero_point: i32,
}

// TODO remove default values, they should be taken from the config file
impl Default for ConfigValues {
    fn default() -> Self {
        Self {
            shunt_resistor: ElectricalResistance::new::<units::si::electrical_resistance::ohm>(
                100.0,
            ),
            gain: 20.0,
            zero_point: 2048,
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
                    config.shunt_resistor = ElectricalResistance::new::<milliohm>(10.0);
                    config.gain = 10.0;
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
            shunt_resistor: ElectricalResistance::new::<milliohm>(100.0),
            gain: 20.0,
            zero_point: 2048,
        }
    }
}
