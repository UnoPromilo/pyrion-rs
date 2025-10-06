use crate::advanced_adc::injected::configured::{Configured, Continuous, ConversionMode, Single};
use crate::advanced_adc::injected::pac::{ModifyPac, ReadPac};
use crate::advanced_adc::pac::RegManipulations;
use crate::advanced_adc::{AdcInstance, EndOfConversionSignal};
use embassy_stm32::adc::AnyAdcChannel;
use embassy_stm32::interrupt::typelevel::Interrupt;
use shared::trace;
use stm32_metapac::adc::vals::SampleTime;

pub struct Running<I: AdcInstance, C: ConversionMode, const CHANNELS: usize> {
    configured: Configured<I, C>,
    channels: [AnyAdcChannel<I>; CHANNELS],
}

impl<I: AdcInstance, const CHANNELS: usize> Running<I, Continuous, CHANNELS> {
    pub(crate) fn new(
        configured: Configured<I, Continuous>,
        values: [(AnyAdcChannel<I>, SampleTime); CHANNELS],
    ) -> Self {
        let instance = Self::inner_new(configured, values);
        I::start();

        instance
    }

    /// Reads the value of the latest conversion
    pub fn read_now(&self) -> [u16; CHANNELS] {
        let mut values = [0; CHANNELS];
        for i in 0..CHANNELS {
            values[i] = I::read_value(i);
        }
        values
    }

    pub async fn read_next(&self) -> [u16; CHANNELS] {
        let result = I::state().jeos_signal.wait().await;
        let mut values = [0; CHANNELS];
        for i in 0..CHANNELS {
            values[i] = result[i]
        }
        values
    }
}

impl<I: AdcInstance, const CHANNELS: usize> Running<I, Single, CHANNELS> {
    pub(crate) fn new(
        configured: Configured<I, Single>,
        values: [(AnyAdcChannel<I>, SampleTime); CHANNELS],
    ) -> Self {
        Self::inner_new(configured, values)
    }

    pub async fn trigger_and_read(&self) -> [u16; CHANNELS] {
        I::start();
        let result = I::state().jeos_signal.wait().await;
        // TODO do I need to stop?
        I::stop();

        let mut values = [0; CHANNELS];
        for i in 0..CHANNELS {
            values[i] = result[i]
        }
        values
    }
}

impl<I: AdcInstance, C: ConversionMode, const CHANNELS: usize> Running<I, C, CHANNELS> {
    pub fn release(self) -> (Configured<I, C>, [AnyAdcChannel<I>; CHANNELS]) {
        I::stop();
        I::set_end_of_conversion_signal_injected(EndOfConversionSignal::None);
        (self.configured, self.channels)
    }

    fn inner_new(
        configured: Configured<I, C>,
        values: [(AnyAdcChannel<I>, SampleTime); CHANNELS],
    ) -> Self {
        // TODO move set_length to proper file or unify
        I::set_length(CHANNELS as u8 - 1);
        for (index, (channel, sample_time)) in values.iter().enumerate() {
            // TODO refine messages?
            trace!(
                "Registering interrupted channel {} (index: {}) with sample time {:?}",
                channel.get_hw_channel(),
                index,
                sample_time
            );
            I::set_channel_sample_time(channel, *sample_time);
            I::register_channel(channel, index);
        }
        let channels = values.map(|(ch, _)| ch);

        I::clear_end_of_conversion_signal_injected(EndOfConversionSignal::Both);
        I::set_end_of_conversion_signal_injected(EndOfConversionSignal::Sequence);
        unsafe { I::Interrupt::enable() }

        Self {
            configured,
            channels,
        }
    }
}
