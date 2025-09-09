use shared::units::{Current, Resistance, Voltage};

const SHIFT: u8 = 32;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy)]
pub struct ConversionConstants {
    adc_to_mv_scale: u64,
    gain_scale: u64,
    v_ref_mid_mv: i64,
}

pub fn calculate_scaling_constants(
    v_ref: Voltage,
    shunt_resistor: Resistance,
    gain: u8,
) -> ConversionConstants {
    let v_ref_mv = v_ref.as_millivolts() as u64;
    let shunt_mohm = shunt_resistor.as_milliohms() as u64;
    let adc_to_mv_scale = (v_ref_mv << SHIFT) / 4095;
    let gain_scale = ((1000u64) << SHIFT) / (gain as u64 * shunt_mohm);

    ConversionConstants {
        adc_to_mv_scale,
        gain_scale,
        v_ref_mid_mv: (v_ref_mv / 2) as i64,
    }
}

#[inline(always)]
pub fn from_adc_to_current(raw: u16, constants: &ConversionConstants) -> Current {
    let v_mv_fp = (raw as u64 * constants.adc_to_mv_scale) >> SHIFT;
    let v_diff_mv = v_mv_fp as i64 - constants.v_ref_mid_mv;
    let current_ma = (v_diff_mv * constants.gain_scale as i64) >> SHIFT;
    Current::from_milliamps(current_ma.clamp(i32::MIN as i64, i32::MAX as i64) as i32)
}

impl ConversionConstants {
    pub fn recalculate_mid_value(&mut self, raw_zero: u16) {
        let zero = (raw_zero as u64 * self.adc_to_mv_scale) >> SHIFT;
        let difference = self.v_ref_mid_mv - zero as i64;
        self.v_ref_mid_mv -= difference;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::units::{Resistance, Voltage};

    #[test]
    fn test_convert_mid_scale() {
        let input = 2048; // Roughly half of 4095
        let constants = calculate_scaling_constants(
            Voltage::from_millivolts(3300),
            Resistance::from_milliohms(100),
            20,
        );
        let result = from_adc_to_current(input, &constants);

        // At mid-scale, v_out ≈ 0, so Current should be ≈ 0
        assert_eq!(result, Current::from_milliamps(0));
    }

    #[test]
    fn test_convert_positive_current() {
        let input = 3000; // Above mid-scale
        let constants = calculate_scaling_constants(
            Voltage::from_millivolts(3300),
            Resistance::from_milliohms(100),
            20,
        );
        let result = from_adc_to_current(input, &constants);

        // Manual calc:
        // v_out = (3000 * 3300 / 4095) - 1650 ≈ 767 mV
        // i = 767 * 1000 / (100 * 20) ≈ 43 mA
        assert_eq!(result, Current::from_milliamps(383));
    }

    #[test]
    fn test_convert_negative_current() {
        let input = 1000; // Below mid-scale
        let constants = calculate_scaling_constants(
            Voltage::from_millivolts(5000),
            Resistance::from_milliohms(10),
            40,
        );
        let result = from_adc_to_current(input, &constants);

        // v_out = (1000 * 5000 / 4095) - 2500 ≈ -1279 mV
        // i = -1279 * 1000 / (10 * 40) ≈ -3198 mA
        assert_eq!(result, Current::from_milliamps(-3198));
    }

    #[test]
    fn test_convert_absolute_max_current() {
        let input = 4095;
        let constants = calculate_scaling_constants(
            Voltage::from_millivolts(3300),
            Resistance::from_milliohms(100),
            20,
        );
        let result = from_adc_to_current(input, &constants);

        // v_out = (4095 * 3300 / 4095) - 1650 = 1650 mV
        // i = 1650 * 1000 / (100 * 20) = 825 mA
        assert_eq!(result, Current::from_milliamps(824));
    }

    #[test]
    fn test_convert_absolute_min_current() {
        let input = 0;
        let constants = calculate_scaling_constants(
            Voltage::from_millivolts(3300),
            Resistance::from_milliohms(100),
            20,
        );
        let result = from_adc_to_current(input, &constants);

        // v_out = (0 * 3300 / 4095) - 1650 = -1650 mV
        // i = 1650 * 1000 / (10 * 40) = -825 mA
        assert_eq!(result, Current::from_milliamps(-825));
    }

    #[test]
    fn test_recalculate_mid_value() {
        let new_zero = 2000; // We have an offset -48 that we need to compensate
        let mut constants = calculate_scaling_constants(
            Voltage::from_millivolts(3300),
            Resistance::from_milliohms(100),
            20,
        );
        constants.recalculate_mid_value(new_zero);

        // At new zero, v_out ≈ 0, so Current should be ≈ 0
        let result = from_adc_to_current(new_zero, &constants);
        assert_eq!(result, Current::from_milliamps(0));
    }
}
