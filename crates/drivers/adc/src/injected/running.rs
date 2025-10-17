use crate::injected::Configured;
use crate::injected::pac::{ModifyPac, ReadPac};
use crate::{AdcInstance, Continuous, EndOfConversionSignal, Single};
use defmt::debug;
use embassy_stm32::adc::AnyAdcChannel;
use embassy_stm32::interrupt::typelevel::Interrupt;
use logging::trace;
use stm32_metapac::adc::vals::SampleTime;

pub struct Running<I: AdcInstance, C, const CHANNELS: usize> {
    configured: Configured<I, C>,
    channels: [AnyAdcChannel<I>; CHANNELS],
}

impl<I: AdcInstance, const CHANNELS: usize> Running<I, Single, CHANNELS> {
    pub(crate) fn new(
        configured: Configured<I, Single>,
        sequence: [(AnyAdcChannel<I>, SampleTime); CHANNELS],
    ) -> Self {
        Self::inner_new(configured, sequence)
    }

    pub async fn trigger_and_read(&self) -> [u16; CHANNELS] {
        I::start();
        let result = I::state().jeos_signal.wait().await;
        I::stop();

        let mut values = [0; CHANNELS];
        values[..CHANNELS].copy_from_slice(&result[..CHANNELS]);
        values
    }
}

impl<I: AdcInstance, const CHANNELS: usize> Running<I, Continuous, CHANNELS> {
    pub(crate) fn new(
        configured: Configured<I, Continuous>,
        sequence: [(AnyAdcChannel<I>, SampleTime); CHANNELS],
    ) -> Self {
        let instance = Self::inner_new(configured, sequence);
        if !I::regs().cfgr().read().jauto() {
            debug!("Injected {} started", I::get_name(),);
            I::start();
        } else {
            debug!("Injected {} started in auto mode", I::get_name(),);
        }
        instance
    }

    /// Reads the value of the latest conversion
    pub fn read_now(&self) -> [u16; CHANNELS] {
        let mut values = [0; CHANNELS];
        for (i, item) in values.iter_mut().enumerate() {
            *item = I::read_value(i);
        }
        values
    }

    pub async fn read_next(&self) -> [u16; CHANNELS] {
        let result = I::state().jeos_signal.wait().await;
        let mut values = [0; CHANNELS];
        values.copy_from_slice(&result[..CHANNELS]);
        values
    }
}

impl<I: AdcInstance, C, const CHANNELS: usize> Running<I, C, CHANNELS> {
    fn inner_new(
        configured: Configured<I, C>,
        sequence: [(AnyAdcChannel<I>, SampleTime); CHANNELS],
    ) -> Self {
        I::set_length(CHANNELS as u8 - 1);
        for (index, (channel, sample_time)) in sequence.iter().enumerate() {
            trace!(
                "Registering interrupted channel {} (index: {}) with sample time {:?}",
                channel.get_hw_channel(),
                index,
                sample_time
            );
            I::set_channel_sample_time(channel, *sample_time);
            I::register_channel(channel, index);
        }
        let channels = sequence.map(|(ch, _)| ch);

        I::clear_end_of_conversion_signal(EndOfConversionSignal::Both);
        I::set_end_of_conversion_signal(EndOfConversionSignal::Sequence);
        unsafe { I::Interrupt::enable() }

        Self {
            configured,
            channels,
        }
    }
    pub fn release(self) -> (Configured<I, C>, [AnyAdcChannel<I>; CHANNELS]) {
        I::stop();
        I::set_end_of_conversion_signal(EndOfConversionSignal::None);
        (self.configured, self.channels)
    }
}
