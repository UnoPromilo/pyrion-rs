use crate::advanced_adc::injected::AnyExtTrigger;
use crate::advanced_adc::injected::config::Config;
use crate::advanced_adc::injected::pac::ModifyPac;
use crate::advanced_adc::injected::running::Running;
use crate::advanced_adc::pac::RegManipulations;
use crate::advanced_adc::{AdcInstance, EndOfConversionSignal, InterruptHandler};
use core::marker::PhantomData;
use embassy_stm32::adc::AnyAdcChannel;
use embassy_stm32::interrupt::typelevel::Binding;
use stm32_metapac::adc::vals::SampleTime;

// TODO a macro?
pub trait ConversionMode {}
pub struct Single;
pub struct Continuous;
impl ConversionMode for Single {}
impl ConversionMode for Continuous {}

pub mod channels {
    pub struct ConstU<const N: usize>;

    pub trait Channels {}
    macro_rules! impl_channels {
        ($($n:literal),*) => {
            $(impl Channels for ConstU<$n> {})*
        };
    }
    impl_channels!(1, 2, 3, 4);
}

pub struct Configured<I: AdcInstance, C: ConversionMode> {
    _phantom: PhantomData<(I, C)>,
}

impl<I: AdcInstance> Configured<I, Continuous> {
    pub(crate) fn new_triggered(trigger: AnyExtTrigger, config: Config) -> Self {
        I::set_ext_trigger(trigger);
        I::set_discontinuous_mode(false);
        I::set_auto_conversion_mode(false);
        I::clear_end_of_conversion_signal_injected(EndOfConversionSignal::Both);
        I::set_end_of_conversion_signal_injected(EndOfConversionSignal::Sequence);
        Self {
            _phantom: PhantomData,
        }
    }

    pub(crate) fn new_auto(config: Config) -> Self {
        I::set_software_trigger();
        I::set_discontinuous_mode(false);
        I::set_auto_conversion_mode(true);
        I::clear_end_of_conversion_signal_injected(EndOfConversionSignal::Both);
        I::set_end_of_conversion_signal_injected(EndOfConversionSignal::Sequence);
        Self {
            _phantom: PhantomData,
        }
    }

    pub fn start<const CHANNELS: usize>(
        self,
        values: [(AnyAdcChannel<I>, SampleTime); CHANNELS],
        _irq: impl Binding<I::Interrupt, InterruptHandler<I>>,
    ) -> Running<I, Continuous, CHANNELS>
    where
        channels::ConstU<CHANNELS>: channels::Channels,
    {
        Running::<I, Continuous, CHANNELS>::new(self, values)
    }
}
impl<I: AdcInstance> Configured<I, Single> {
    pub(crate) fn new_single(config: Config) -> Self {
        I::set_software_trigger();
        I::set_discontinuous_mode(false);
        I::set_auto_conversion_mode(false);
        Self {
            _phantom: PhantomData,
        }
    }

    pub fn prepare<const CHANNELS: usize>(
        self,
        values: [(AnyAdcChannel<I>, SampleTime); CHANNELS],
        _irq: impl Binding<I::Interrupt, InterruptHandler<I>>,
    ) -> Running<I, Single, CHANNELS>
    where
        channels::ConstU<CHANNELS>: channels::Channels,
    {
        Running::<I, Single, CHANNELS>::new(self, values)
    }
}
