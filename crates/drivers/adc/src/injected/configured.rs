use crate::injected::AnyExtTrigger;
use crate::injected::pac::ModifyPac;
use crate::injected::running::Running;
use crate::{
    AdcInstance, Continuous, EndOfConversionSignal, InterruptHandler, Single, define_channels_mod,
};
use core::marker::PhantomData;
use embassy_stm32::adc::AnyAdcChannel;
use embassy_stm32::interrupt::typelevel::Binding;
use logging::trace;
use stm32_metapac::adc::vals::SampleTime;

define_channels_mod!(channels, [1, 2, 3, 4]);

pub struct Configured<I: AdcInstance, C> {
    _phantom: PhantomData<(I, C)>,
}

impl<I: AdcInstance> Configured<I, Single> {
    #[allow(dead_code)]
    pub(crate) fn new_single() -> Self {
        I::set_software_trigger();
        I::set_discontinuous_mode(false);
        I::set_auto_conversion_mode(false);
        Self {
            _phantom: PhantomData,
        }
    }

    #[allow(dead_code)]
    pub fn prepare<const CHANNELS: usize>(
        self,
        sequence: [(AnyAdcChannel<I>, SampleTime); CHANNELS],
        _irq: impl Binding<I::Interrupt, InterruptHandler<I>>,
    ) -> Running<I, Single, CHANNELS>
    where
        channels::ConstU<CHANNELS>: channels::Channels,
    {
        Running::<I, Single, CHANNELS>::new(self, sequence)
    }
}

impl<I: AdcInstance> Configured<I, Continuous> {
    #[allow(dead_code)]
    pub(crate) fn new_triggered(trigger: AnyExtTrigger) -> Self {
        trace!(
            "Configuring injected {} triggered on {}",
            I::get_name(),
            trigger
        );
        I::set_ext_trigger(trigger);
        I::set_discontinuous_mode(false);
        I::set_auto_conversion_mode(false);
        I::clear_end_of_conversion_signal(EndOfConversionSignal::Both);
        I::set_end_of_conversion_signal(EndOfConversionSignal::Sequence);
        Self {
            _phantom: PhantomData,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn new_auto() -> Self {
        trace!("Configuring injected {} auto", I::get_name(),);
        I::set_software_trigger();
        I::set_discontinuous_mode(false);
        I::set_auto_conversion_mode(true);
        I::clear_end_of_conversion_signal(EndOfConversionSignal::Both);
        I::set_end_of_conversion_signal(EndOfConversionSignal::Sequence);
        Self {
            _phantom: PhantomData,
        }
    }

    pub fn start<const CHANNELS: usize>(
        self,
        sequence: [(AnyAdcChannel<I>, SampleTime); CHANNELS],
        _irq: impl Binding<I::Interrupt, InterruptHandler<I>>,
    ) -> Running<I, Continuous, CHANNELS>
    where
        channels::ConstU<CHANNELS>: channels::Channels,
    {
        Running::<I, Continuous, CHANNELS>::new(self, sequence)
    }
}
