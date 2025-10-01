use crate::advanced_adc::injected::config::Config;
use crate::advanced_adc::injected::pac::ModifyPac;
use crate::advanced_adc::injected::running::Running;
use crate::advanced_adc::injected::triggers::Trigger;
use crate::advanced_adc::pac_instance::PacInstance;
use core::marker::PhantomData;
use embassy_stm32::adc::AnyAdcChannel;
use stm32_metapac::adc::vals::SampleTime;

// TODO a macro?
pub mod channels {
    pub struct ConstU<const N: usize>;

    pub trait Channels {}
    impl Channels for ConstU<1> {}
    impl Channels for ConstU<2> {}
    impl Channels for ConstU<3> {}
    impl Channels for ConstU<4> {}
}

pub struct Configured<T: PacInstance> {
    _phantom: PhantomData<T>,
}

impl<I: PacInstance> Configured<I> {
    pub(crate) fn new<T: Trigger>(config: Config<T>) -> Self {
        I::set_trigger(config.trigger);
        I::set_auto_conversion_mode(false);
        Self {
            _phantom: PhantomData,
        }
    }

    pub fn start<const CHANNELS: usize>(
        self,
        values: [(AnyAdcChannel<I>, SampleTime); CHANNELS],
    ) -> Running<I, CHANNELS>
    where
        channels::ConstU<CHANNELS>: channels::Channels,
    {
        I::set_length(CHANNELS as u8);
        // if CHANNELS == 1 { todo!()}
        for (index, (channel, sample_time)) in values.iter().enumerate() {
            I::set_channel_sample_time(channel, *sample_time);
            I::register_channel(channel, index);
        }

        I::start();
        Running::new(self)
    }

    pub(crate) fn downgrade<const N: usize>(_val: Running<I, N>) -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}
