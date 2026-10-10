use crate::adc::AdcInstance;
use crate::adc::injected::pac::ModifyPac;
use crate::adc::injected::running::Running;
use crate::adc::interrupt::InterruptHandler;
use core::marker::PhantomData;
use embassy_stm32::adc::AnyAdcChannel;
use embassy_stm32::interrupt::typelevel::Binding;
use logging::debug;
use stm32_metapac::adc::vals::SampleTime;

pub struct Configured<I: AdcInstance> {
    _phantom: PhantomData<I>,
}

impl<I: AdcInstance> Configured<I> {
    pub(crate) fn new_tim1_triggered() -> Self {
        debug!("Configuring injected {} on TIM1_TRGO", I::get_name());
        I::set_tim1_trigger();
        I::set_discontinuous_mode(false);
        I::set_auto_conversion_mode(false);
        Self {
            _phantom: PhantomData,
        }
    }

    pub fn start<H>(
        self,
        sequence: [(AnyAdcChannel<I>, SampleTime); 2],
        _irq: impl Binding<I::Interrupt, H>,
    ) -> Running<I>
    where
        H: InterruptHandler<I::Interrupt>,
    {
        Running::new(self, sequence)
    }
}
