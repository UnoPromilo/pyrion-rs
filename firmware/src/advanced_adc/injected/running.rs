use crate::advanced_adc::AdcInstance;
use crate::advanced_adc::injected::configured;
use crate::advanced_adc::injected::configured::Configured;
use crate::advanced_adc::injected::pac::{ModifyPac, ReadPac};
use embassy_stm32::adc::AnyAdcChannel;

pub struct Running<I: AdcInstance, const CHANNELS: usize> {
    configured: Configured<I, configured::Continuous>,
    channels: [AnyAdcChannel<I>; CHANNELS],
}

impl<I: AdcInstance, const CHANNELS: usize> Running<I, CHANNELS> {
    pub(crate) fn new(
        configured: Configured<I, configured::Continuous>,
        channels: [AnyAdcChannel<I>; CHANNELS],
    ) -> Self {
        Self {
            configured,
            channels,
        }
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
        todo!("wait for the next conversion, then read")
    }

    pub fn stop(self) -> Configured<I, configured::Continuous> {
        I::stop();
        self.configured
    }

    pub fn release(
        self,
    ) -> (
        Configured<I, configured::Continuous>,
        [AnyAdcChannel<I>; CHANNELS],
    ) {
        (self.configured, self.channels)
    }
}
