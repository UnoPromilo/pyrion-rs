use crate::advanced_adc::regular::Config;
use crate::advanced_adc::regular::pac::ModifyPac;
use crate::advanced_adc::regular::running::Running;
use crate::advanced_adc::{AdcInstance, InterruptHandler, Single};
use core::marker::PhantomData;
use embassy_stm32::adc::AnyAdcChannel;
use embassy_stm32::interrupt::typelevel::Binding;
use stm32_metapac::adc::vals::SampleTime;

pub mod channels {
    pub struct ConstU<const N: usize>;

    pub trait Channels {}
    macro_rules! impl_channels {
        ($($n:literal),*) => {
            $(impl Channels for ConstU<$n> {})*
        };
    }
    impl_channels!(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16);
}

pub struct Configured<I: AdcInstance, C> {
    _phantom: PhantomData<(I, C)>,
}

impl<'d, I: AdcInstance> Configured<I, Single> {
    pub(crate) fn new_single(config: Config) -> Self {
        I::set_software_trigger();
        I::set_continuous_mode(config.enable_continuous_mode);
        I::set_discontinuous_mode(false);
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
