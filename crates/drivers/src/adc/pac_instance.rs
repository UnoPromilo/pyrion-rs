use embassy_stm32::adc::Instance;

pub trait PacInstance: Instance {
    fn regs() -> stm32_metapac::adc::Adc;
}

macro_rules! impl_pac_instance {
    ($($inst:ident),* $(,)?) => {
        $(
            impl PacInstance for embassy_stm32::peripherals::$inst {
                #[inline(always)]
                fn regs() -> stm32_metapac::adc::Adc {
                    stm32_metapac::$inst
                }
            }
        )*
    };
}

impl_pac_instance!(ADC1, ADC3, ADC5,);
