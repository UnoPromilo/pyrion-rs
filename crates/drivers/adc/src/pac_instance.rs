use embassy_stm32::adc::Instance;

pub trait PacInstance: Instance {
    fn regs() -> stm32_metapac::adc::Adc;
    fn common_regs() -> stm32_metapac::adccommon::AdcCommon;
}

macro_rules! impl_pac_instance {
    ($($inst:ident => $common:ident),* $(,)?) => {
        $(
            impl PacInstance for embassy_stm32::peripherals::$inst {
                #[inline(always)]
                fn regs() -> stm32_metapac::adc::Adc {
                    stm32_metapac::$inst
                }

                #[inline(always)]
                fn common_regs() -> stm32_metapac::adccommon::AdcCommon {
                    stm32_metapac::$common
                }
            }
        )*
    };
}

impl_pac_instance!(
    ADC1 => ADC12_COMMON,
    ADC2 => ADC12_COMMON,
    ADC3 => ADC345_COMMON,
    ADC4 => ADC345_COMMON,
    ADC5 => ADC345_COMMON,
);
