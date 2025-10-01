use embassy_stm32::adc::Instance;

pub trait PacInstance: Instance {
    fn regs() -> stm32_metapac::adc::Adc;
    fn common_regs() -> stm32_metapac::adccommon::AdcCommon;
}

macro_rules! foreach_adc {
    ($($pat:tt => $code:tt;)*) => {
        macro_rules! __inner {
            $(($pat) => $code;)*
            ($_:tt) => {}
        }
        __inner!((ADC1,ADC12_COMMON));
        __inner!((ADC2,ADC12_COMMON));
        __inner!((ADC3,ADC345_COMMON));
        __inner!((ADC4,ADC345_COMMON));
        __inner!((ADC5,ADC345_COMMON));
    };
}

foreach_adc!(
    ($inst:ident, $common:ident) => {
        impl PacInstance for embassy_stm32::peripherals::$inst {
            fn regs() -> stm32_metapac::adc::Adc {
                stm32_metapac::$inst
            }

            fn common_regs() -> stm32_metapac::adccommon::AdcCommon {
                stm32_metapac::$common
            }
        }
    };
);
