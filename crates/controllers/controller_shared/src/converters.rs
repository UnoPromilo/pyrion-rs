use units::si::electric_potential::millivolt;
use units::{ElectricCurrent, ElectricPotential, ElectricalResistance};

const VREF_CALIB_MV: f32 = 3000.0;
const ADC_MAX_COUNT: f32 = 4095.0;

pub fn current_per_adc_count(
    vrefint_sample: u16,
    vrefint_cal: u16,
    scales: &BoardSensorScales,
) -> ElectricCurrent {
    let voltage = convert_to_voltage(1, vrefint_sample, vrefint_cal);
    voltage / scales.current_gain / scales.shunt_resistance
}

pub fn convert_to_current(
    sample: u16,
    zero_offset: u16,
    current_per_count: ElectricCurrent,
) -> ElectricCurrent {
    current_per_count * (sample as i32 - zero_offset as i32) as f32
}

pub fn convert_to_voltage(sample: i32, vrefint_sample: u16, vrefint_cal: u16) -> ElectricPotential {
    let vrefint_mv = VREF_CALIB_MV * vrefint_cal as f32 / ADC_MAX_COUNT;
    ElectricPotential::new::<millivolt>(sample as f32 * vrefint_mv / vrefint_sample as f32)
}

#[derive(Clone, Copy, Debug)]
pub struct BoardSensorScales {
    pub shunt_resistance: ElectricalResistance,
    pub current_gain: f32,
    pub v_bus_scale_ratio: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use units::si::electric_current::milliampere;
    use units::si::electric_potential::volt;
    use units::si::electrical_resistance::milliohm;
    const VREFINT: u16 = 1550;
    const VREFINT_CAL: u16 = 1652;

    #[test]
    fn test_convert_to_current() {
        struct TestCase {
            sample: u16,
            vrefint: u16,
            expected: f32,
            config: BoardSensorScales,
        }

        let test_cases = [
            TestCase {
                sample: 2048,
                vrefint: VREFINT,
                expected: 0.0,
                config: test_scales(),
            },
            TestCase {
                sample: 2486,
                vrefint: VREFINT,
                expected: 170.9975,
                config: test_scales(),
            },
            TestCase {
                sample: 1622,
                vrefint: VREFINT,
                expected: -166.3127,
                config: test_scales(),
            },
            TestCase {
                sample: 1622,
                vrefint: VREFINT,
                expected: -3326.2532,
                config: {
                    let mut config = test_scales();
                    config.shunt_resistance = ElectricalResistance::new::<milliohm>(10.0);
                    config.current_gain = 10.0;
                    config
                },
            },
        ];

        for test_case in test_cases {
            let per_count =
                current_per_adc_count(test_case.vrefint, VREFINT_CAL, &test_case.config);
            let result = convert_to_current(test_case.sample, 2048, per_count);
            let raw_result = result.get::<milliampere>();
            assert!(
                (raw_result - test_case.expected).abs() < 0.05,
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
                sample: 4095,
                vrefint: VREFINT,
                expected: 3197.4194,
            },
            TestCase {
                sample: -1,
                vrefint: VREFINT,
                expected: -0.78081,
            },
        ];

        for test_case in test_cases {
            let result = convert_to_voltage(test_case.sample, test_case.vrefint, VREFINT_CAL);
            let raw_result = result.get::<millivolt>();
            assert!(
                (raw_result - test_case.expected).abs() < 0.01,
                "Expected {}mV, got {}mV, error: {}mV",
                test_case.expected,
                raw_result,
                raw_result - test_case.expected
            )
        }
    }

    #[test]
    fn factory_reference_calibration_sets_voltage_scale() {
        let nominal = convert_to_voltage(1000, VREFINT, VREFINT_CAL).get::<millivolt>();
        let lower = convert_to_voltage(1000, VREFINT, 1600).get::<millivolt>();

        assert!(lower < nominal);
        assert!((lower / nominal - 1600.0 / VREFINT_CAL as f32).abs() < 0.0001);
    }

    #[test]
    fn bus_voltage_applies_configured_scale_to_calibrated_adc_voltage() {
        let adc_voltage = convert_to_voltage(1000, VREFINT, VREFINT_CAL);
        let bus_voltage = adc_voltage * test_scales().v_bus_scale_ratio;

        assert!((bus_voltage.get::<volt>() - 1.5616).abs() < 0.001);
        let scaled_bus_voltage = adc_voltage * (test_scales().v_bus_scale_ratio * 2.0);
        assert!(
            (scaled_bus_voltage.get::<volt>() - 2.0 * bus_voltage.get::<volt>()).abs() < 0.0001
        );
    }

    #[test]
    fn current_zero_offset_is_applied_before_conversion() {
        let config = test_scales();
        let per_count = current_per_adc_count(VREFINT, VREFINT_CAL, &config);
        let zero = convert_to_current(2073, 2073, per_count);
        let below = convert_to_current(2072, 2073, per_count);
        let above = convert_to_current(2074, 2073, per_count);

        assert_eq!(zero.get::<milliampere>(), 0.0);
        assert!(below.get::<milliampere>() < 0.0);
        assert!(above.get::<milliampere>() > 0.0);
    }

    fn test_scales() -> BoardSensorScales {
        BoardSensorScales {
            shunt_resistance: ElectricalResistance::new::<milliohm>(100.0),
            current_gain: 20.0,
            v_bus_scale_ratio: 2.0,
        }
    }
}
