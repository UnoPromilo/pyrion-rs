use embassy_stm32::time::Hertz;
use stm32_metapac::adccommon::vals::Presc;

const MAX_ADC_CLK_FREQ: Hertz = Hertz::mhz(60);

pub trait Prescaler {
    fn from_kernel_clock(frequency: Hertz) -> Self;

    fn divisor(&self) -> u32;
}

impl Prescaler for Presc {
    fn from_kernel_clock(frequency: Hertz) -> Self {
        let raw_prescaler = frequency.0 / MAX_ADC_CLK_FREQ.0;
        match raw_prescaler {
            0 => Presc::DIV1,
            1 => Presc::DIV2,
            2..=3 => Presc::DIV4,
            4..=5 => Presc::DIV6,
            6..=7 => Presc::DIV8,
            8..=9 => Presc::DIV10,
            10..=11 => Presc::DIV12,
            _ => unimplemented!(),
        }
    }

    fn divisor(&self) -> u32 {
        match self {
            Presc::DIV1 => 1,
            Presc::DIV2 => 2,
            Presc::DIV4 => 4,
            Presc::DIV6 => 6,
            Presc::DIV8 => 8,
            Presc::DIV10 => 10,
            Presc::DIV12 => 12,
            _ => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn from_kernel_clock_test_should_never_return_more_than_max_adc_clk_freq() {
        let max_freq = MAX_ADC_CLK_FREQ;
        for i in 0..340 {
            let freq = Hertz::mhz(i);
            let prescaler = Presc::from_kernel_clock(freq);
            assert!(prescaler.divisor() <= max_freq.0);
        }
    }
}
