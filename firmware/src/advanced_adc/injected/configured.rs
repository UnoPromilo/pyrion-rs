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
        Self {
            _phantom: PhantomData,
        }
    }

    pub(crate) fn new_auto(config: Config) -> Self {
        I::set_software_trigger();
        I::set_discontinuous_mode(false);
        I::set_auto_conversion_mode(true);
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

        // TODO clear isr.ovr?
        I::start();
        let channels = values.map(|(ch, _)| ch);
        Running::new(self, channels)
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

    pub async fn read<const CHANNELS: usize>(
        &mut self,
        values: [(&mut AnyAdcChannel<I>, SampleTime); CHANNELS],
        _irq: impl Binding<I::Interrupt, InterruptHandler<I>>,
    ) -> [u16; CHANNELS]
    where
        channels::ConstU<CHANNELS>: channels::Channels,
    {
        I::set_length(CHANNELS as u8);
        // if CHANNELS == 1 { todo!()}
        for (index, (channel, sample_time)) in values.iter().enumerate() {
            I::set_channel_sample_time(channel, *sample_time);
            I::register_channel(channel, index);
        }

        // TODO clear isr.ovr?
        // TODO set jeosen
        I::clear_end_of_conversion_signal_injected(EndOfConversionSignal::Both);
        I::set_end_of_conversion_signal_injected(EndOfConversionSignal::Sequence);
        I::start();
        let result = I::state().jeos_signal.wait().await;

        let mut output = [0; CHANNELS];
        for (index, value) in result.iter().enumerate() {
            output[index] = *value;
        }

        I::stop();
        // reset
        I::set_end_of_conversion_signal_injected(EndOfConversionSignal::None);
        output
    }
}
